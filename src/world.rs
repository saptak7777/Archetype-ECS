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
use std::any::TypeId;
use std::ptr::NonNull;

#[cfg(feature = "profiling")]
use tracing::info_span;

use crate::archetype::{Archetype, ArchetypeSignature};
use crate::command::CommandBuffer;
use crate::component::{Bundle, Component, MAX_BUNDLE_COMPONENTS};
use crate::entity::{EntityId, EntityLocation};
use crate::error::{EcsError, Result};
use crate::query::{QueryFetch, QueryFetchMut, QueryFilter, QueryMut};

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

    /// Cache for type IDs to avoid repeated allocations
    type_id_cache: AHashMap<Vec<TypeId>, usize>,
}

impl World {
    /// Create new world
    pub fn new() -> Self {
        let mut world = Self {
            entity_locations: SlotMap::with_key(),
            recycled_entities: 0,
            archetypes: Vec::with_capacity(32), // Pre-allocate some capacity
            archetype_index: AHashMap::with_capacity(32),
            transitions: AHashMap::with_capacity(128), // Pre-allocate for common transitions
            type_id_cache: AHashMap::with_capacity(32),
        };

        // Create empty archetype for entities with no components
        world.get_or_create_archetype(&[]); // FIXED: Pass vec directly
        world
    }

    /// Spawn entity with components
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Result<EntityId> {
        self.ensure_entity_capacity();
        let placeholder = EntityLocation {
            archetype_id: usize::MAX,
            archetype_row: usize::MAX,
        };
        let entity = self.entity_locations.insert(placeholder);
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

        // Get or create archetype for this component set, registering component columns only once
        let archetype_id = self.get_or_create_archetype_with(type_ids.as_slice(), |archetype| {
            B::register_components(archetype);
            archetype.mark_columns_initialized();
        });
        let archetype = &mut self.archetypes[archetype_id];

        // Allocate row in archetype
        let row = archetype.allocate_row(entity);

        // Write component data
        let mut ptrs = [std::ptr::null_mut(); MAX_BUNDLE_COMPONENTS];
        let mut ptr_count = 0;
        for &type_id in type_ids.iter() {
            if let Some(column) = archetype.get_column_mut(type_id) {
                ptrs[ptr_count] = column.get_ptr_mut(row);
                ptr_count += 1;
            }
        }

        unsafe {
            bundle.write_components(&ptrs[..ptr_count]);
        }

        // Update entity location
        if let Some(loc) = self.entity_locations.get_mut(entity) {
            *loc = EntityLocation {
                archetype_id,
                archetype_row: row,
            };
        }

        Ok(entity)
    }

    /// Despawn entity
    pub fn despawn(&mut self, entity: EntityId) -> Result<()> {
        if let Some(location) = self.entity_locations.remove(entity) {
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
        } else {
            Err(EcsError::EntityNotFound)
        }
    }

    /// Get entity location
    pub fn get_entity_location(&self, entity: EntityId) -> Option<EntityLocation> {
        self.entity_locations.get(entity).copied()
    }

    /// Get immutable reference to a component on an entity
    pub fn get_component<T: Component>(&self, entity: EntityId) -> Option<&T> {
        let location = self.entity_locations.get(entity)?;
        let archetype = self.archetypes.get(location.archetype_id)?;
        let column = archetype.get_column(TypeId::of::<T>())?;
        column.get::<T>(location.archetype_row)
    }

    /// Get mutable reference to a component on an entity
    pub fn get_component_mut<T: Component>(&mut self, entity: EntityId) -> Option<&mut T> {
        let location = self.entity_locations.get(entity)?;
        let archetype = self.archetypes.get_mut(location.archetype_id)?;
        let column = archetype.get_column_mut(TypeId::of::<T>())?;
        column.get_mut::<T>(location.archetype_row)
    }

    /// Get multiple immutable components at once using QueryFetch
    pub fn get_components<'a, Q>(&'a self, entity: EntityId) -> Option<<Q as QueryFetch<'a>>::Item>
    where
        Q: QueryFetch<'a>,
    {
        let location = self.entity_locations.get(entity)?;
        let archetype = self.archetypes.get(location.archetype_id)?;
        Q::fetch(archetype, location.archetype_row)
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
        Q::fetch_mut(archetype, location.archetype_row)
    }

    /// Create a mutable query wrapper for the provided filter
    pub fn query_mut<'w, Q>(&'w mut self) -> QueryMut<'w, Q>
    where
        Q: QueryFilter + QueryFetchMut<'w>,
    {
        QueryMut::new(self)
    }

    /// Check if entity exists
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
    pub(crate) fn archetype_ptr_mut(&mut self, id: usize) -> Option<NonNull<Archetype>> {
        self.archetypes.get_mut(id).map(NonNull::from)
    }

    /// Get archetype count
    pub fn archetype_count(&self) -> usize {
        self.archetypes.len()
    }

    /// Get total entity count
    pub fn entity_count(&self) -> u32 {
        self.entity_locations.len() as u32
    }

    /// Get recycled entity count
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

    /// Get or create archetype with caching for common signatures
    fn get_or_create_archetype(&mut self, signature: &[TypeId]) -> usize {
        self.get_or_create_archetype_with(signature, |_| {})
    }

    /// Get or create archetype with a callback for initialization
    fn get_or_create_archetype_with<F>(&mut self, signature: &[TypeId], on_create: F) -> usize
    where
        F: FnOnce(&mut Archetype),
    {
        // Convert to owned Vec for lookup
        let signature_vec = signature.to_vec();

        // Try to find in archetype_index first (more direct than cache)
        if let Some(&id) = self.archetype_index.get(&signature_vec) {
            // Update cache
            self.type_id_cache.entry(signature_vec).or_insert(id);
            return id;
        }

        // Not found, create new archetype
        let id = self.archetypes.len();

        // Create new archetype with the signature
        let mut archetype = Archetype::new(signature_vec.clone());
        on_create(&mut archetype);

        // Store in archetype index and cache
        self.archetype_index.insert(signature_vec.clone(), id);
        self.type_id_cache.insert(signature_vec, id);
        self.archetypes.push(archetype);

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
        if count == 0 {
            return Ok(Vec::new());
        }

        // Ensure we have enough capacity
        if self.entity_locations.len() + count > self.entity_locations.capacity() {
            let additional = (self.entity_locations.capacity() + count).max(1024);
            self.entity_locations.reserve(additional);
        }

        // Get or create archetype first
        let type_ids = B::type_ids();
        let archetype_id = self.get_or_create_archetype_with(type_ids.as_slice(), |archetype| {
            B::register_components(archetype);
            archetype.mark_columns_initialized();
        });

        // Get mutable reference to archetype after all lookups are done
        let archetype = &mut self.archetypes[archetype_id];
        let mut entity_ids = Vec::with_capacity(count);

        // Pre-allocate space in the archetype
        archetype.reserve_rows(count);

        // Process each bundle
        for bundle in bundles {
            let entity = self.entity_locations.insert(EntityLocation {
                archetype_id,
                archetype_row: 0, // Will be updated after allocation
            });

            // Allocate row in archetype
            let row = archetype.allocate_row(entity);

            // Update entity location with correct row
            if let Some(loc) = self.entity_locations.get_mut(entity) {
                loc.archetype_row = row;
            }

            // Write component data
            let mut ptrs = [std::ptr::null_mut(); MAX_BUNDLE_COMPONENTS];
            let mut ptr_count = 0;
            for &type_id in type_ids.iter() {
                if let Some(column) = archetype.get_column_mut(type_id) {
                    ptrs[ptr_count] = column.get_ptr_mut(row);
                    ptr_count += 1;
                }
            }

            unsafe {
                bundle.write_components(&ptrs[..ptr_count]);
            }

            entity_ids.push(entity);
        }

        Ok(entity_ids)
    }

    /// Ensure we have enough capacity for new entities with an aggressive growth strategy
    fn ensure_entity_capacity(&mut self) {
        let current_cap = self.entity_locations.capacity();
        if self.entity_locations.len() >= current_cap {
            // Use exponential growth with a minimum of 1024
            let additional = (current_cap * 2).max(1024);
            self.entity_locations.reserve(additional);
        }
    }
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

        #[derive(Debug)]
        struct Position {
            x: f32,
            y: f32,
        }

        let entity = world.spawn((Position { x: 1.0, y: 2.0 },))?;
        assert!(world.get_entity_location(entity).is_some());

        world.despawn(entity)?;
        assert!(world.get_entity_location(entity).is_none());

        Ok(())
    }

    #[test]
    fn test_archetype_segregation() -> Result<()> {
        let mut world = World::new();

        struct A;
        struct B;
        struct C;

        world.spawn((A, B))?;
        world.spawn((A, C))?;
        world.spawn((B, C))?;

        // Should create 4 archetypes (+ empty one)
        assert!(world.archetype_count() >= 4);

        Ok(())
    }
}
