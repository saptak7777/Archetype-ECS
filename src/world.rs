// Copyright 2024 Saptak Santra
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! World: central entity and archetype storage

use ahash::AHashMap;
use slotmap::SlotMap;
use smallvec::SmallVec;
use std::any::TypeId;
use std::cell::RefCell;
use std::ptr::NonNull;

#[cfg(feature = "profiling")]
use tracing::info_span;

use crate::archetype::{Archetype, ArchetypeSignature};
use crate::command::CommandBuffer;
use crate::component::{Bundle, Component, MAX_BUNDLE_COMPONENTS};
use crate::entity::{EntityId, EntityLocation};
use crate::error::{EcsError, Result};
use crate::event::{EntityEvent, EventQueue};
use crate::observer::{Observer, ObserverRegistry};
use crate::query::{Query, QueryFetch, QueryFetchMut, QueryFilter, QueryMut};

/// Central ECS world
/// The World is the central type that holds all entities, components, and systems.
pub struct World {
    /// Entity locations keyed by SlotMap IDs
    entity_locations: SlotMap<EntityId, EntityLocation>,

    /// Recycled entity counter (for diagnostics)
    recycled_entities: usize,

    /// All archetypes in the world
    archetypes: Vec<Archetype>,

    /// Maps component type signatures to archetype indices
    archetype_index: AHashMap<ArchetypeSignature, usize>,

    /// Cache for archetype transitions when adding/removing components
    transitions: AHashMap<(usize, TypeId, bool), usize>,

    /// Event queue for deferred event processing
    event_queue: EventQueue,

    /// Observer registry for lifecycle events
    observers: ObserverRegistry,

    /// Component tracker for change detection
    component_tracker: AHashMap<EntityId, std::collections::HashSet<TypeId>>,

    /// Global event bus for pub/sub communication (Phase 6)
    global_event_bus: crate::event_bus::EventBus,

    /// Current world tick
    tick: u32,

    /// Deferred removal queue for safe entity deletion during iteration
    removal_queue: Vec<EntityId>,

    /// Typed resources (singletons) for global state
    resources: AHashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,

    /// Query result cache to avoid O(n) archetype scanning
    /// Maps generic Query type ID to QueryState
    query_cache: RefCell<AHashMap<crate::query::QuerySignature, crate::query::CachedQueryResult>>,
}

impl World {
    /// Create a new, empty world.
    pub fn new() -> Self {
        let mut world = Self {
            entity_locations: SlotMap::with_key(),
            recycled_entities: 0,

            // Start with reasonable defaults to avoid resize spikes
            archetypes: Vec::with_capacity(64),
            archetype_index: AHashMap::with_capacity(64),
            transitions: AHashMap::with_capacity(128),

            // Subsystems
            event_queue: EventQueue::new(),
            observers: ObserverRegistry::new(),
            component_tracker: AHashMap::new(),
            global_event_bus: crate::event_bus::EventBus::new(),

            tick: 1, // Tick 0 is reserved/unused to ensure change detection checks always pass for new things
            removal_queue: Vec::new(),
            resources: AHashMap::new(),
            // Pre-allocate query cache - trades memory for speed (most apps have <100 unique queries)
            query_cache: RefCell::new(AHashMap::with_capacity(32)),
        };

        // Bootstrap the empty archetype (entities with no components)
        // This is always at index 0 and simplifies logic elsewhere
        world.get_or_create_archetype_with(&ArchetypeSignature::new(), |arch| {
            arch.mark_columns_initialized();
        });
        world
    }

    pub fn tick(&self) -> u32 {
        self.tick
    }

    pub fn increment_tick(&mut self) {
        // Panic on overflow - tick wraparound would break change detection
        if self.tick == u32::MAX {
            panic!("World tick overflow at {}", self.tick);
        }
        self.tick = self.tick.wrapping_add(1);
    }

    /// Spawn entity with components
    /// Spawn a new entity with the given bundle of components.
    ///
    /// # Panics
    /// Panics if the Entity ID generator overflows (which is practically impossible).
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> EntityId {
        // Ensure capacity before insertion (panic on overflow is acceptable)
        self.ensure_entity_capacity();

        let placeholder = EntityLocation {
            archetype_id: usize::MAX,
            archetype_row: usize::MAX,
        };

        let id = self.entity_locations.insert(placeholder);

        if self.recycled_entities > 0 {
            self.recycled_entities -= 1;
        }
        let type_ids = B::type_ids();
        #[cfg(feature = "profiling")]
        let span = info_span!(
            "world.spawn",
            bundle_components = type_ids.len(),
            archetype_count = self.archetypes.len()
        );
        #[cfg(feature = "profiling")]
        let _span_guard = span.enter();

        let arch_id = self.get_or_create_archetype_with(&type_ids, |archetype| {
            B::register_components(archetype);
            archetype.mark_columns_initialized();
        });
        let archetype = &mut self.archetypes[arch_id];

        // Allocate row in archetype
        let row = archetype.allocate_row(id, self.tick);

        // OPTIMIZATION: Pre-calculate column indices to avoid hash lookups
        let mut column_indices = [usize::MAX; MAX_BUNDLE_COMPONENTS];
        let mut column_count = 0;
        for &type_id in type_ids.iter() {
            if let Some(idx) = archetype.column_index(type_id) {
                column_indices[column_count] = idx;
                column_count += 1;
            }
        }

        // Write component data using pre-calculated indices
        let mut ptrs = [std::ptr::null_mut(); MAX_BUNDLE_COMPONENTS];
        for i in 0..column_count {
            let col_idx = column_indices[i];
            if let Some(column) = archetype.get_column_mut_by_index(col_idx) {
                ptrs[i] = column.get_ptr_mut(row);
            }
        }

        unsafe {
            bundle.write_components(&ptrs[..column_count]);
        }

        // Update entity location
        // Note: SlotMap insert happened earlier to get ID, now we update value
        if let Some(loc) = self.entity_locations.get_mut(id) {
            *loc = EntityLocation {
                archetype_id: arch_id,
                archetype_row: row,
            };
        }

        // Track components
        let mut component_set = std::collections::HashSet::with_capacity(type_ids.len());
        for &type_id in type_ids.iter() {
            component_set.insert(type_id);
        }
        self.component_tracker.insert(id, component_set);

        // Return entity ID
        id
    }

    /// Check if an entity is alive
    ///
    /// Returns true if the entity handle is valid and the entity exists in the world.
    pub fn is_alive(&self, entity: EntityId) -> bool {
        self.entity_locations.contains_key(entity)
    }

    /// Despawn entity (deferred - queued for removal)
    ///
    /// Entities are not immediately removed to avoid issues during iteration.
    /// Call `flush_removals()` to process the removal queue.
    pub fn despawn_deferred(&mut self, entity: EntityId) -> Result<()> {
        // Validate entity exists
        if !self.entity_locations.contains_key(entity) {
            return Err(EcsError::EntityNotFound);
        }

        // Queue for deferred removal
        self.removal_queue.push(entity);
        Ok(())
    }

    /// Despawn entity immediately
    ///
    /// Removes the entity and all its components from the world.
    pub fn despawn(&mut self, entity: EntityId) -> Result<()> {
        // Fail fast on invalid entity
        if !self.entity_locations.contains_key(entity) {
            return Err(EcsError::EntityNotFound);
        }

        let location = self.entity_locations.remove(entity).unwrap();
        let archetype = &mut self.archetypes[location.archetype_id];
        unsafe {
            if let Some(swapped_entity) = archetype.remove_row(location.archetype_row) {
                if let Some(swapped_loc) = self.entity_locations.get_mut(swapped_entity) {
                    swapped_loc.archetype_row = location.archetype_row;
                }
            }
        }
        self.recycled_entities += 1;
        Ok(())
    }

    /// Flush deferred removal queue
    ///
    /// Processes all entities queued for removal in batch.
    /// This is more efficient than removing one-by-one and safe during iteration.
    pub fn flush_removals(&mut self) -> Result<()> {
        let to_remove: Vec<_> = self.removal_queue.drain(..).collect();

        if to_remove.is_empty() {
            return Ok(());
        }

        // Validate first removal to catch queue corruption early
        let first = to_remove[0];
        if !self.entity_locations.contains_key(first) {
            return Err(EcsError::EntityNotFound);
        }

        self.despawn(first)?;

        // Subsequent entities may be duplicates or already removed (e.g., cascading despawns)
        // Skip gracefully to avoid crashing on valid scenarios
        for &entity in &to_remove[1..] {
            if self.entity_locations.contains_key(entity) {
                let _ = self.despawn(entity);
            }
        }

        Ok(())
    }

    /// Get entity location
    pub fn get_entity_location(&self, entity: EntityId) -> Option<EntityLocation> {
        self.entity_locations.get(entity).copied()
    }

    /// Get immutable reference to a component on an entity
    pub fn get_component<T: Component>(&self, entity: EntityId) -> Option<&T> {
        // Returns None for invalid entity - simpler API, caller decides error handling
        let location = self.entity_locations.get(entity)?;
        let archetype = self.archetypes.get(location.archetype_id)?;
        let column = archetype.get_column(TypeId::of::<T>())?;
        column.get::<T>(location.archetype_row)
    }

    /// Get mutable reference to a component on an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        // BOUNDARY: Validate entity exists before component lookup
        let location = self.entity_locations.get(entity)?;
        let tick = self.tick;
        let archetype = self.archetypes.get_mut(location.archetype_id)?;
        let column = archetype.get_column_mut(TypeId::of::<T>())?;

        // Mark component as changed for change detection
        column.mark_changed(location.archetype_row, tick);

        column.get_mut::<T>(location.archetype_row)
    }

    /// Check if entity has a specific component
    pub fn has_component<T: Component>(&self, entity: EntityId) -> bool {
        if let Some(location) = self.entity_locations.get(entity) {
            if let Some(archetype) = self.archetypes.get(location.archetype_id) {
                return archetype.has_column(TypeId::of::<T>());
            }
        }
        false
    }

    /// Add a component to an entity
    ///
    /// This is an expensive operation as it moves the entity to a new archetype.
    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) -> Result<()> {
        let location = *self
            .entity_locations
            .get(entity)
            .ok_or(EcsError::EntityNotFound)?;
        let old_archetype = &mut self.archetypes[location.archetype_id];

        // If component already exists, overwrite it
        if let Some(col) = old_archetype.get_column_mut(TypeId::of::<T>()) {
            let ptr = col.get_ptr_mut(location.archetype_row) as *mut T;
            unsafe {
                std::ptr::write(ptr, component);
            }
            return Ok(());
        }

        // Calculate new signature
        let mut new_signature = old_archetype.signature().clone();
        new_signature.push(TypeId::of::<T>());

        // Capture existing columns to replicate them in new archetype
        // We need to do this before calling get_or_create_archetype as that requires mutable self access,
        // which would conflict with holding a reference to old_archetype.
        let mut columns_to_add = Vec::with_capacity(new_signature.len());
        for &type_id in old_archetype.signature() {
            if let Some(col) = old_archetype.get_column(type_id) {
                columns_to_add.push((type_id, col.clone_empty()));
            }
        }

        let new_archetype_id = self.get_or_create_archetype_with(&new_signature, |archetype| {
            for (type_id, col) in columns_to_add {
                archetype.add_column_raw(type_id, col);
            }
            archetype.register_component::<T>();
            archetype.mark_columns_initialized();
        });

        // Move entity
        self.move_entity(entity, location, new_archetype_id, |archetype, row| {
            // Initialize new component
            if let Some(col) = archetype.get_column_mut(TypeId::of::<T>()) {
                let ptr = col.get_ptr_mut(row) as *mut T;
                unsafe {
                    std::ptr::write(ptr, component);
                }
            }
        })
    }

    /// Remove a component from an entity
    ///
    /// This is an expensive operation as it moves the entity to a new archetype.
    pub fn remove_component<T: Component>(&mut self, entity: EntityId) -> Result<()> {
        let old_location = self
            .entity_locations
            .get(entity)
            .copied()
            .ok_or(EcsError::EntityNotFound)?;
        let old_archetype = &self.archetypes[old_location.archetype_id];

        // PRE-CONDITION: Verify component exists on entity
        let component_type_id = TypeId::of::<T>();
        if !old_archetype.has_column(component_type_id) {
            return Err(EcsError::ComponentNotFound);
        }

        // Build new signature (excluding component T)
        let mut new_signature = old_archetype.signature().clone();
        new_signature.retain(|tid| *tid != component_type_id);

        // Capture existing columns to replicate them in new archetype.
        // This must be done before we potentially push to self.archetypes.
        let mut columns_to_add = Vec::with_capacity(new_signature.len());
        for &type_id in &new_signature {
            if let Some(col) = old_archetype.get_column(type_id) {
                columns_to_add.push((type_id, col.clone_empty()));
            }
        }

        let new_archetype_id = self.get_or_create_archetype_with(&new_signature, |new_arch| {
            for (type_id, col) in columns_to_add {
                new_arch.add_column_raw(type_id, col);
            }
            new_arch.mark_columns_initialized();
        });

        // POST-CONDITION: Verify destination archetype is ready
        #[cfg(debug_assertions)]
        {
            let arch = &self.archetypes[new_archetype_id];
            debug_assert!(
                arch.columns_initialized(),
                "BUG: Destination archetype columns not initialized"
            );
            for &tid in arch.signature() {
                debug_assert!(
                    arch.has_column(tid),
                    "BUG: Destination archetype missing column for type {tid:?}"
                );
            }
        }

        // Safe migration: move entity and drop the removed component implicitly
        self.move_entity(entity, old_location, new_archetype_id, |_, _| {})
    }

    /// Get multiple immutable components at once using QueryFetch
    pub fn get_components<'a, Q>(&'a self, entity: EntityId) -> Option<<Q as QueryFetch<'a>>::Item>
    where
        Q: QueryFetch<'a>,
    {
        let location = self.entity_locations.get(entity)?;
        let archetype = self.archetypes.get(location.archetype_id)?;
        let state = Q::prepare(archetype, 0)?;
        unsafe { Q::fetch(&state, location.archetype_row) }
    }

    /// Get multiple mutable components at once using QueryFetchMut
    pub fn get_components_mut<'a, Q>(
        &'a mut self,
        entity: EntityId,
    ) -> Option<<Q as QueryFetchMut<'a>>::Item>
    where
        Q: QueryFetchMut<'a>,
    {
        let location = self.entity_locations.get(entity)?;
        let archetype = self.archetypes.get_mut(location.archetype_id)?;
        let mut state = Q::prepare(archetype, 0, self.tick)?;
        unsafe { Q::fetch(&mut state, location.archetype_row) }
    }

    /// Create a mutable query wrapper for the provided filter
    pub fn query_mut<'w, Q>(&'w mut self) -> QueryMut<'w, Q>
    where
        Q: QueryFilter + QueryFetchMut<'w>,
    {
        QueryMut::new(self)
    }

    pub fn query<'w, Q>(&'w self) -> Query<'w, Q>
    where
        Q: QueryFilter + QueryFetch<'w>,
    {
        Query::new(self)
    }

    /// Create a parallel query wrapper for the provided filter
    ///
    /// Requires the "parallel" feature.
    #[cfg(feature = "parallel")]
    pub fn par_query_mut<'w, Q>(&'w mut self) -> crate::query::ParQuery<'w, Q>
    where
        Q: QueryFilter + QueryFetchMut<'w>,
    {
        crate::query::ParQuery::new(self.query_mut())
    }

    /// Internal: Move entity from one archetype to another
    fn move_entity<F>(
        &mut self,
        entity: EntityId,
        old_loc: EntityLocation,
        new_archetype_id: usize,
        on_new_location: F,
    ) -> Result<()>
    where
        F: FnOnce(&mut Archetype, usize),
    {
        if old_loc.archetype_id == new_archetype_id {
            return Ok(());
        }

        let tick = self.tick;
        // We need to ensure new archetype has space (it does via allocate_row logic usually, but let's be safe if reserve needed)
        // actually allocate_row just pushes.

        // Access both archetypes safely using split_at_mut
        // We need this to copy components from old to new.
        let (old_arch, new_arch) = if old_loc.archetype_id < new_archetype_id {
            let (left, right) = self.archetypes.split_at_mut(new_archetype_id);
            (&mut left[old_loc.archetype_id], &mut right[0])
        } else {
            let (left, right) = self.archetypes.split_at_mut(old_loc.archetype_id);
            (&mut right[0], &mut left[new_archetype_id])
        };

        // Allocate row in new archetype
        let new_row = new_arch.allocate_row(entity, tick);

        unsafe {
            let new_sig = new_arch.signature().to_vec();

            for &type_id in &new_sig {
                if let Some(old_col) = old_arch.get_column_mut(type_id) {
                    if let Some(new_col) = new_arch.get_column_mut(type_id) {
                        let src = old_col.get_ptr_mut(old_loc.archetype_row);
                        let dst = new_col.get_ptr_mut(new_row);
                        // Copy raw bytes
                        std::ptr::copy_nonoverlapping(src, dst, old_col.get_item_size());
                    }
                }
            }
        }

        on_new_location(new_arch, new_row);

        // Remove from old archetype
        unsafe {
            if let Some(swapped_entity) = old_arch.remove_row(old_loc.archetype_row) {
                if let Some(swapped_loc_ptr) = self.entity_locations.get_mut(swapped_entity) {
                    swapped_loc_ptr.archetype_row = old_loc.archetype_row;
                }
            }
        }

        // Update location of moved entity
        if let Some(loc) = self.entity_locations.get_mut(entity) {
            loc.archetype_id = new_archetype_id;
            loc.archetype_row = new_row;
        }

        Ok(())
    }

    /// Get cached query results (matched archetypes)
    ///
    /// This method manages the query cache, updating it incrementally if needed.
    /// It returns a vector of archetype indices that match the query.
    /// It returns a vector of archetype indices that match the query.
    pub(crate) fn get_cached_query_indices<Q: QueryFilter>(&self) -> Vec<usize> {
        let sig = Q::signature();

        // Fast path: existing state
        {
            let mut cache = self.query_cache.borrow_mut();
            if let Some(cached) = cache.get_mut(&sig) {
                cached.update(&self.archetypes);
                // Clone to avoid lifetime issues with mutable cache access
                return cached.matches.to_vec();
            }
        }

        // Slow path: create new state
        let cached = crate::query::CachedQueryResult::new(sig.clone(), &self.archetypes);
        let indices = cached.matches.to_vec();
        self.query_cache.borrow_mut().insert(sig, cached);
        indices
    }

    pub fn entity_exists(&self, entity: EntityId) -> bool {
        self.entity_locations.contains_key(entity)
    }

    /// Get archetype by ID
    pub fn get_archetype(&self, id: usize) -> Option<&Archetype> {
        self.archetypes.get(id)
    }

    /// Get archetype mutably
    pub fn get_archetype_mut(&mut self, id: usize) -> Option<&mut Archetype> {
        self.archetypes.get_mut(id)
    }

    /// Get all archetypes
    pub fn archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    /// Internal helper to expose archetype pointers for query iteration
    pub(crate) fn archetype_ptr(&self, id: usize) -> Option<NonNull<Archetype>> {
        self.archetypes.get(id).map(NonNull::from)
    }

    /// Internal helper to expose archetype pointers for query iteration
    ///
    /// # Safety
    /// Returned pointer is valid for the lifetime of the world.
    /// Caller must ensure no aliasing violations when dereferencing.
    pub(crate) fn archetype_ptr_mut(&mut self, id: usize) -> Option<NonNull<Archetype>> {
        self.archetypes.get_mut(id).map(NonNull::from)
    }

    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    pub fn entity_count(&self) -> u32 {
        self.entity_locations.len() as u32
    }

    pub fn recycled_entity_count(&self) -> usize {
        self.recycled_entities
    }

    /// Flush command buffer
    pub fn flush_commands(&mut self, buffer: CommandBuffer) -> Result<()> {
        #[cfg(feature = "profiling")]
        let span = info_span!("world.flush_commands", queued = buffer.len());
        #[cfg(feature = "profiling")]
        let _span_guard = span.enter();

        for command in buffer.into_iter() {
            // FIXED: Use into_iter()
            match command {
                crate::command::Command::Spawn { bundle_fn } => {
                    bundle_fn(self)?;
                }
                crate::command::Command::Despawn(entity) => {
                    // FIXED: Tuple variant
                    self.despawn(entity)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.entity_locations.clear();
        self.recycled_entities = 0;
        self.archetypes.clear();
        self.archetype_index.clear();
        self.transitions.clear();
        self.query_cache.borrow_mut().clear();

        // Recreate empty archetype
        self.get_or_create_archetype(&[]); // FIXED
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        let archetype_memory: usize = self
            .archetypes
            .iter()
            .map(|_a| std::mem::size_of::<Archetype>()) // FIXED: _a
            .sum();
        let entity_index_memory =
            self.entity_locations.capacity() * std::mem::size_of::<EntityLocation>();

        MemoryStats {
            entity_index_memory,
            archetype_memory,
            total_memory: archetype_memory + entity_index_memory,
        }
    }

    // ========== Resource API (Singleton State) ==========

    /// Insert a resource (singleton) into the world
    ///
    /// Resources are typed singletons that can be accessed globally.
    /// If a resource of this type already exists, it will be replaced.
    ///
    /// # Example
    /// ```ignore
    /// world.insert_resource(Time { delta: 0.016 });
    /// ```
    pub fn insert_resource<R: Send + Sync + 'static>(&mut self, resource: R) {
        self.resources.insert(TypeId::of::<R>(), Box::new(resource));
    }

    /// Get an immutable reference to a resource
    ///
    /// Returns `None` if the resource doesn't exist.
    pub fn resource<R: 'static>(&self) -> Option<&R> {
        self.resources
            .get(&TypeId::of::<R>())
            .and_then(|r| r.downcast_ref())
    }

    /// Get a mutable reference to a resource
    ///
    /// Returns `None` if the resource doesn't exist.
    pub fn resource_mut<R: 'static>(&mut self) -> Option<&mut R> {
        self.resources
            .get_mut(&TypeId::of::<R>())
            .and_then(|r| r.downcast_mut())
    }

    /// Check if a resource exists
    pub fn has_resource<R: 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<R>())
    }

    /// Remove a resource and return it
    pub fn remove_resource<R: 'static>(&mut self) -> Option<R> {
        self.resources
            .remove(&TypeId::of::<R>())
            .and_then(|r| r.downcast().ok())
            .map(|boxed| *boxed)
    }

    /// Get or create archetype with caching for common signatures
    fn get_or_create_archetype(&mut self, signature: &[TypeId]) -> usize {
        // PARANOID: Prevent archetype explosion DoS attack
        if self.archetypes.len() >= 10_000 {
            panic!("Archetype limit exceeded (10,000) - possible DoS attempt or memory leak");
        }
        let signature_vec: ArchetypeSignature = SmallVec::from_slice(signature);
        self.get_or_create_archetype_with(&signature_vec, |_| {})
    }

    /// Get or create archetype with a callback for initialization
    fn get_or_create_archetype_with<F>(
        &mut self,
        signature: &ArchetypeSignature,
        on_create: F,
    ) -> usize
    where
        F: FnOnce(&mut Archetype),
    {
        // Sort signature to ensure canonical lookup (prevent archetype fragmentation)
        // This ensures that (A, B) and (B, A) map to the same archetype logic
        let mut sorted_signature = signature.clone();
        sorted_signature.sort();

        // Try to find in archetype_index first (more direct than cache)
        if let Some(&id) = self.archetype_index.get(&sorted_signature) {
            return id;
        }

        // Not found, create new archetype

        // Create new archetype with the sorted signature
        let mut archetype = Archetype::new(sorted_signature.clone());
        on_create(&mut archetype);

        // Push archetype FIRST to ensure it exists
        self.archetypes.push(archetype);
        let id = self.archetypes.len() - 1;

        // THEN cache the ID (prevents returning non-existent IDs)
        self.archetype_index.insert(sorted_signature, id);

        id
    }

    /// Spawn multiple entities with the same component bundle in a batch
    ///
    /// This is more efficient than calling `spawn` multiple times as it reduces
    /// the number of allocations and lookups.
    pub fn spawn_batch<B, I>(&mut self, bundles: I) -> Result<Vec<EntityId>>
    where
        B: Bundle,
        I: IntoIterator<Item = B>,
        I::IntoIter: ExactSizeIterator,
    {
        let bundles = bundles.into_iter();
        let count = bundles.len();

        // Limit batch size to prevent OOM and overflow (10M is generous but prevents DoS)
        if count > 10_000_000 {
            return Err(EcsError::BatchTooLarge);
        }

        if count == 0 {
            return Ok(Vec::new());
        }

        // Saturating add: prefer capped growth over overflow panic
        let current = self.entity_locations.len();
        let new_capacity = current.saturating_add(count).max(1024);

        if current + count > self.entity_locations.capacity() {
            let additional = new_capacity - current;
            self.entity_locations.reserve(additional);
        }

        // Get or create archetype first
        let type_ids = B::type_ids();
        let archetype_id = self.get_or_create_archetype_with(&type_ids, |archetype| {
            B::register_components(archetype);
            archetype.mark_columns_initialized();
        });

        // Get mutable reference to archetype after all lookups are done
        let archetype = &mut self.archetypes[archetype_id];
        let mut entity_ids = Vec::with_capacity(count);

        // Pre-allocate space in the archetype
        archetype.reserve_rows(count);

        // OPTIMIZATION: Pre-calculate column indices to avoid hash lookups in the hot loop
        let mut column_indices = [usize::MAX; MAX_BUNDLE_COMPONENTS];
        let mut col_count = 0;
        for &tid in type_ids.iter() {
            if let Some(idx) = archetype.column_index(tid) {
                column_indices[col_count] = idx;
                col_count += 1;
            }
        }

        // Process each bundle
        for bundle in bundles {
            let entity = self.entity_locations.insert(EntityLocation {
                archetype_id,
                archetype_row: 0, // Will be updated after allocation
            });

            // Allocate row in archetype
            let row = archetype.allocate_row(entity, self.tick);

            // Update entity location with correct row
            if let Some(loc) = self.entity_locations.get_mut(entity) {
                loc.archetype_row = row;
            }

            // Write component data using pre-calculated indices
            let mut ptrs = [std::ptr::null_mut(); MAX_BUNDLE_COMPONENTS];
            for i in 0..col_count {
                let col_idx = column_indices[i];
                if let Some(column) = archetype.get_column_mut_by_index(col_idx) {
                    ptrs[i] = column.get_ptr_mut(row);
                }
            }

            unsafe {
                bundle.write_components(&ptrs[..col_count]);
            }

            entity_ids.push(entity);
        }

        Ok(entity_ids)
    }

    /// Ensure we have enough capacity for new entities with an aggressive growth strategy
    fn ensure_entity_capacity(&mut self) {
        let len = self.entity_locations.len();

        // Panic on overflow - indicates programming error, not user error
        if len >= usize::MAX - 1024 {
            panic!("Entity ID exhaustion: {len:#x} entities allocated");
        }

        let cap = self.entity_locations.capacity();

        if len >= cap {
            // Aggressive growth to reduce reallocations
            let growth = (cap / 2).max(64);
            self.entity_locations.reserve(growth);
        }
    }

    /// Spawn entity with components and trigger event
    pub fn spawn_with_event<B: Bundle>(&mut self, bundle: B) -> EntityId {
        let entity = self.spawn(bundle);
        self.event_queue.push(EntityEvent::Spawned(entity));

        // Track components for this entity
        let type_ids = B::type_ids();
        let mut components = std::collections::HashSet::new();
        for &type_id in type_ids.iter() {
            components.insert(type_id);
            self.event_queue
                .push(EntityEvent::ComponentAdded(entity, type_id));
        }
        self.component_tracker.insert(entity, components);

        entity
    }

    /// Despawn entity and trigger event
    pub fn despawn_with_event(&mut self, entity: EntityId) -> Result<()> {
        self.despawn(entity)?;
        self.event_queue.push(EntityEvent::Despawned(entity));
        self.component_tracker.remove(&entity);
        Ok(())
    }

    /// Register observer
    pub fn register_observer(&mut self, mut observer: Box<dyn Observer>) -> Result<()> {
        // Call on_registered before storing
        observer.on_registered(self)?;
        self.observers.observers.push(observer);
        Ok(())
    }

    /// Process all pending events
    pub fn process_events(&mut self) -> Result<()> {
        // We need to work around Rust's borrow checker here.
        // We can't borrow event_queue and observers simultaneously since both are in self.
        // Solution: drain events into a temporary vector, then process with unsafe aliasing.
        let mut events_to_process = Vec::new();
        while let Some(event) = self.event_queue.pop() {
            events_to_process.push(event);
        }

        // Use unsafe to allow observers to access world (self) while we iterate observers
        // This is safe because:
        // 1. We're not modifying the observers vector itself during iteration
        // 2. Observers are expected to only read/write to specific parts of World
        // 3. This is similar to the parallel executor pattern
        let world_ptr = self as *mut World;

        for event in &events_to_process {
            for observer in &mut self.observers.observers {
                unsafe {
                    observer.on_event(event, &mut *world_ptr)?;
                }
            }
        }
        Ok(())
    }

    /// Manually trigger event
    pub fn trigger_event(&mut self, event: EntityEvent) {
        self.event_queue.push(event);
    }

    /// Get observer registry
    pub fn observers_mut(&mut self) -> &mut ObserverRegistry {
        &mut self.observers
    }

    /// Get event queue (for inspection)
    pub fn event_queue(&self) -> &EventQueue {
        &self.event_queue
    }

    // ========== Hierarchy Methods (Phase 5) ==========

    /// Get parent of entity
    pub fn get_parent(&self, entity: EntityId) -> Option<EntityId> {
        use crate::hierarchy::Parent;
        self.get_component::<Parent>(entity).map(|p| p.entity_id())
    }

    /// Get children of entity
    pub fn get_children(&self, entity: EntityId) -> Option<Vec<EntityId>> {
        use crate::hierarchy::Children;
        self.get_component::<Children>(entity)
            .map(|c| c.get_children())
    }

    /// Traverse hierarchy depth-first
    pub fn traverse_hierarchy<F>(&self, entity: EntityId, callback: &mut F) -> Result<()>
    where
        F: FnMut(EntityId) -> Result<()>,
    {
        use crate::hierarchy::Children;

        callback(entity)?;

        if let Some(children) = self.get_component::<Children>(entity) {
            for &child in children.iter() {
                self.traverse_hierarchy(child, callback)?;
            }
        }

        Ok(())
    }

    /// Get all descendants of entity
    pub fn get_descendants(&self, entity: EntityId) -> Result<Vec<EntityId>> {
        let mut descendants = Vec::new();

        self.traverse_hierarchy(entity, &mut |e| {
            if e != entity {
                // Don't include the entity itself
                descendants.push(e);
            }
            Ok(())
        })?;

        Ok(descendants)
    }

    /// Delete entity and all children recursively
    pub fn despawn_recursive(&mut self, entity: EntityId) -> Result<()> {
        // Get children before despawning
        let children = self.get_children(entity).unwrap_or_default();

        // Recursively despawn children
        for child in children {
            self.despawn_recursive(child)?;
        }

        // Despawn this entity
        self.despawn(entity)?;

        Ok(())
    }

    // ========== Global Event Bus Methods (Phase 6) ==========

    /// Get mutable reference to global event bus
    pub fn event_bus_mut(&mut self) -> &mut crate::event_bus::EventBus {
        &mut self.global_event_bus
    }

    /// Get immutable reference to global event bus
    pub fn event_bus(&self) -> &crate::event_bus::EventBus {
        &self.global_event_bus
    }

    /// Publish event to global event bus (convenience method)
    pub fn publish_global_event(&mut self, event: Box<dyn crate::event_bus::Event>) -> Result<()> {
        self.global_event_bus.publish(event)
    }

    /// Process all queued events in global event bus
    pub fn process_global_events(&mut self) -> Result<()> {
        self.global_event_bus.process_events()
    }

    // ========== Query Cache Management (Phase 2) ==========

    /// Get or update cached query results for a signature
    ///
    /// Uses incremental invalidation: only checks new archetypes since last cache update.
    /// This provides O(1) amortized performance for repeated queries.
    pub fn get_cached_query_indices_by_sig(
        &self,
        signature: &crate::query::QuerySignature,
    ) -> Vec<usize> {
        let current_archetype_count = self.archetypes.len();
        let mut cache = self.query_cache.borrow_mut();

        if let Some(cached) = cache.get_mut(signature) {
            if cached.seen_archetypes < current_archetype_count {
                cached.update(&self.archetypes);
            }
            cached.matches.to_vec()
        } else {
            let cached = crate::query::CachedQueryResult::new(signature.clone(), &self.archetypes);
            let indices = cached.matches.to_vec();
            cache.insert(signature.clone(), cached);
            indices
        }
    }

    /// Clear all cached query results
    ///
    /// Useful for testing or when you need to force cache invalidation.
    pub fn clear_query_cache(&self) {
        self.query_cache.borrow_mut().clear();
    }

    /// Get query cache statistics for diagnostics
    /// Get query cache statistics for diagnostics
    pub fn query_cache_stats(&self) -> QueryCacheStats {
        let cache = self.query_cache.borrow();
        let total_cached_archetypes: usize =
            cache.values().map(|cached| cached.matches.len()).sum();

        QueryCacheStats {
            num_cached_queries: cache.len(),
            total_cached_archetypes,
            total_archetypes: self.archetypes.len(),
        }
    }
}

/// Statistics about the query cache
#[derive(Debug, Clone, Copy)]
pub struct QueryCacheStats {
    /// Number of unique query signatures cached
    pub num_cached_queries: usize,
    /// Total number of archetype matches across all cached queries
    pub total_cached_archetypes: usize,
    /// Total number of archetypes in the world
    pub total_archetypes: usize,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory statistics for the world
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub entity_index_memory: usize,
    pub archetype_memory: usize,
    pub total_memory: usize,
}

#[cfg(test)]
mod tests {
    #![allow(dead_code)]
    use super::*;

    #[test]
    fn test_spawn_despawn() -> Result<()> {
        let mut world = World::new();

        let entity = world.spawn((42i32,));
        assert!(world.get_entity_location(entity).is_some());

        world.despawn(entity).unwrap();
        world.flush_removals().unwrap(); // Process deferred removals
        assert!(world.get_entity_location(entity).is_none());
        Ok(())
    }

    #[test]
    fn test_archetype_segregation() -> Result<()> {
        let mut world = World::new();

        struct A;
        struct B;
        struct C;

        world.spawn((A, B));
        world.spawn((A, C));
        world.spawn((B, C));

        // Should create 4 archetypes (+ empty one)
        assert!(world.archetype_count() >= 4);

        Ok(())
    }
}
