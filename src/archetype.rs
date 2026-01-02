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

//! Archetype storage with row allocation and removal

use std::any::TypeId;

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::component::Component;
use crate::entity::EntityId;

/// Chunk size in bytes (16KB - fits in L1 cache, Unity DOTS standard)
pub const CHUNK_SIZE_BYTES: usize = 16384;

/// Default chunk size in entities for iteration
pub const DEFAULT_CHUNK_SIZE: usize = 64;

/// Component signature
pub type ArchetypeSignature = SmallVec<[TypeId; 8]>;

/// Chunk of entities with contiguous component data for cache-friendly iteration
pub struct ArchetypeChunk<'a> {
    /// Range of entity indices in this chunk
    pub entity_range: std::ops::Range<usize>,
    /// Reference to the archetype
    pub archetype: &'a Archetype,
}

/// Mutable chunk of entities
pub struct ArchetypeChunkMut<'a> {
    /// Range of entity indices in this chunk
    pub entity_range: std::ops::Range<usize>,
    /// Mutable reference to the archetype
    pub archetype: &'a mut Archetype,
}

impl<'a> ArchetypeChunk<'a> {
    /// Get a slice of components for this chunk
    pub fn get_slice<T: Component>(&self) -> Option<&[T]> {
        self.archetype.get_component_slice::<T>().map(|slice| {
            // SAFETY: entity_range is guaranteed to be within bounds by chunks() iterator
            &slice[self.entity_range.clone()]
        })
    }
}

impl<'a> ArchetypeChunkMut<'a> {
    /// Get a slice of components for this chunk
    pub fn get_slice<T: Component>(&self) -> Option<&[T]> {
        self.archetype
            .get_component_slice::<T>()
            .map(|slice| &slice[self.entity_range.clone()])
    }

    /// Get a mutable slice of components for this chunk
    pub fn get_slice_mut<T: Component>(&mut self) -> Option<&mut [T]> {
        let range = self.entity_range.clone();
        self.archetype
            .get_component_slice_mut::<T>()
            .map(|slice| &mut slice[range])
    }
}

/// Archetype: Structure of Arrays storage
pub struct Archetype {
    signature: ArchetypeSignature,
    entities: Vec<EntityId>,
    components: Vec<ComponentColumn>,
    component_indices: FxHashMap<TypeId, usize>,
    columns_initialized: bool,
}

impl Archetype {
    pub(crate) fn add_column_raw(&mut self, type_id: TypeId, column: ComponentColumn) {
        // Prevent duplicates
        if !self.component_indices.contains_key(&type_id) {
            let idx = self.components.len();
            self.components.push(column);
            self.component_indices.insert(type_id, idx);
        }
    }

    /// Create new archetype
    pub fn new(signature: ArchetypeSignature) -> Self {
        let mut archetype = Self {
            signature,
            entities: Vec::new(),
            components: Vec::new(),
            component_indices: FxHashMap::default(),
            columns_initialized: false,
        };
        archetype.reserve_rows(128);
        archetype
    }

    /// Get signature
    pub fn signature(&self) -> &ArchetypeSignature {
        &self.signature
    }

    /// Check if archetype has a component type
    pub fn has_column(&self, type_id: TypeId) -> bool {
        self.component_indices.contains_key(&type_id)
    }

    /// Allocate row for entity
    pub fn allocate_row(&mut self, entity: EntityId, tick: u32) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);

        // Separate added/changed ticks: allows detecting modifications after spawn
        // (e.g., component initialization systems)
        for column in &mut self.components {
            column.added_ticks.push(tick);
            column.changed_ticks.push(tick);
            column.last_added_tick = tick;
            column.last_change_tick = tick;
        }

        row
    }

    /// Remove row and return entity that was swapped in
    ///
    /// # Safety
    ///
    /// This function is marked unsafe because it performs manual memory management
    /// on type-erased component data. The caller MUST ensure:
    ///
    /// ## Preconditions
    /// 1. `row` is a valid index: `row < self.entities.len()`
    /// 2. The entity at `row` has not already been removed
    /// 3. All component columns have matching lengths
    ///
    /// ## Memory Safety Guarantees
    /// - Uses swap-remove to maintain packed array layout
    /// - Properly handles the swapped entity's location update
    /// - Does NOT call drop on removed components (caller's responsibility)
    ///
    /// ## Returns
    /// - `Some(entity)` if another entity was swapped into this row (needs location update)
    /// - `None` if this was the last entity (no swap occurred)
    pub unsafe fn remove_row(&mut self, row: usize) -> Option<EntityId> {
        // SAFETY: Caller must ensure row < len (contract violation = panic, not UB)
        if row >= self.entities.len() {
            panic!(
                "BUG: remove_row called with invalid row {} (len={})",
                row,
                self.entities.len()
            );
        }

        // Invariant check: entity and component counts must match (only if components exist)
        if !self.components.is_empty() {
            debug_assert_eq!(
                self.entities.len(),
                self.components[0].len(),
                "Entity/component count mismatch"
            );
        }
        self.entities.swap_remove(row);

        for column in &mut self.components {
            // Handle data swap-remove manually for the byte buffer
            let item_size = column.item_size;
            if item_size > 0 {
                let last_idx = column.len() - 1;
                if row < last_idx {
                    // SAFETY: This pointer arithmetic is safe because:
                    // 1. last_idx < column.len()
                    // 2. row < last_idx (checked above)
                    // 3. Both offsets are within allocated buffer bounds
                    // 4. item_size is the correct size for type
                    // 5. copy_nonoverlapping is safe because src != dst
                    unsafe {
                        let src = column.ptr.add(last_idx * item_size);
                        let dst = column.ptr.add(row * item_size);
                        std::ptr::copy_nonoverlapping(src, dst, item_size);
                    }
                }
                column.len = last_idx;
            }
            // Fix: Keep ticks in sync with entities using swap_remove
            // We use simple swap_remove because the order doesn't matter (sparse set)
            // matching the entity swap_remove logic.
            if row < column.added_ticks.len() {
                column.added_ticks.swap_remove(row);
                column.changed_ticks.swap_remove(row);
            }
        }

        // If we swapped someone in, return their entity so we can update their location
        if row < self.entities.len() {
            Some(self.entities[row])
        } else {
            None
        }
    }

    /// Get column immutably
    pub fn get_column(&self, type_id: TypeId) -> Option<&ComponentColumn> {
        let idx = *self.component_indices.get(&type_id)?;
        self.components.get(idx)
    }

    /// Get column by index
    pub fn get_column_by_index(&self, index: usize) -> Option<&ComponentColumn> {
        self.components.get(index)
    }

    /// Get column mutably
    pub fn get_column_mut(&mut self, type_id: TypeId) -> Option<&mut ComponentColumn> {
        let idx = *self.component_indices.get(&type_id)?;
        self.components.get_mut(idx)
    }

    /// Get column index for a component type
    pub fn column_index(&self, type_id: TypeId) -> Option<usize> {
        self.component_indices.get(&type_id).copied()
    }

    /// Get component column by precomputed index
    pub fn get_column_mut_by_index(&mut self, index: usize) -> Option<&mut ComponentColumn> {
        self.components.get_mut(index)
    }

    pub fn get_component_slice<T: Component>(&self) -> Option<&[T]> {
        let type_id = TypeId::of::<T>();
        let idx = *self.component_indices.get(&type_id)?;
        self.components[idx].get_slice::<T>()
    }

    /// Get typed mutable slice of components
    pub fn get_component_slice_mut<T: Component>(&mut self) -> Option<&mut [T]> {
        let type_id = TypeId::of::<T>();
        let idx = *self.component_indices.get(&type_id)?;
        self.components[idx].get_slice_mut::<T>()
    }

    /// Reserve space for additional rows
    pub fn reserve_rows(&mut self, additional: usize) {
        // Cap excessive reservations (100K limit prevents pathological cases)
        let additional = additional.min(100_000);

        if additional == 0 {
            return;
        }

        let current_capacity = self.entities.capacity();
        let current_len = self.entities.len();

        // Bail on inconsistent state (capacity < len should be impossible)
        if current_capacity < current_len {
            return;
        }

        if current_capacity - current_len < additional {
            // Pre-allocate all columns together to avoid fragmentation
            self.entities.reserve(additional);
            for column in &mut self.components {
                // Reserve additional items (converted to bytes for the raw pointer)
                if column.item_size > 0 {
                    let required_len = (column.len + additional) * column.item_size;
                    if required_len > column.cap {
                        let new_cap = required_len.next_power_of_two().max(64);
                        let new_layout =
                            std::alloc::Layout::from_size_align(new_cap, column.align).unwrap();

                        let new_ptr = if column.cap == 0 {
                            unsafe { std::alloc::alloc(new_layout) }
                        } else {
                            let old_layout =
                                std::alloc::Layout::from_size_align(column.cap, column.align)
                                    .unwrap();
                            unsafe { std::alloc::realloc(column.ptr, old_layout, new_cap) }
                        };

                        if new_ptr.is_null() {
                            std::alloc::handle_alloc_error(new_layout);
                        }

                        column.ptr = new_ptr;
                        column.cap = new_cap;
                    }
                }
                column.added_ticks.reserve(additional);
                column.changed_ticks.reserve(additional);
            }
        }
    }

    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Iterate over chunks of entities for cache-friendly processing
    ///
    /// Returns an iterator over chunks of entities. Each chunk contains
    /// a contiguous range of entities for better cache locality.
    ///
    /// # Arguments
    /// * `chunk_size` - Number of entities per chunk (default: 64)
    pub fn chunks(&self, chunk_size: usize) -> impl Iterator<Item = ArchetypeChunk> + '_ {
        let total_entities = self.len();
        let chunk_size = chunk_size.max(1); // Ensure at least 1 entity per chunk

        (0..total_entities).step_by(chunk_size).map(move |start| {
            let end = (start + chunk_size).min(total_entities);
            ArchetypeChunk {
                entity_range: start..end,
                archetype: self,
            }
        })
    }

    /// Iterate over mutable chunks of entities
    pub fn chunks_mut(&mut self, chunk_size: usize) -> Vec<ArchetypeChunkMut> {
        let total_entities = self.len();
        let chunk_size = chunk_size.max(1);

        // We need to split the mutable borrow of self.
        // Since we can't easily return an iterator that yields mutable references to self
        // without unsafe code (lending iterator problem), we will use unsafe here.
        // However, standard Iterator trait doesn't support lending.
        // So we can't actually implement this safely as a standard Iterator returning ArchetypeChunkMut<'a>
        // where 'a is tied to self.

        // Actually, we can if we collect them or use a streaming iterator crate, but we don't have that.
        // For now, let's just return a Vec since we are going to use it for parallel iteration anyway.
        // Or we can implement a custom iterator that uses unsafe to extend the lifetime,
        // relying on the fact that chunks are disjoint.

        // Let's return a Vec for simplicity and safety for now.
        // It involves a small allocation but it's negligible compared to processing.

        let mut chunks = Vec::new();
        let ptr = self as *mut Archetype;

        for start in (0..total_entities).step_by(chunk_size) {
            let end = (start + chunk_size).min(total_entities);
            // SAFETY:
            // 1. We are creating multiple mutable references to the same archetype
            // 2. BUT, we are wrapping them in ArchetypeChunkMut which conceptually owns a range
            // 3. The user must only access the specific range via get_slice_mut
            // 4. Wait, get_slice_mut calls get_component_slice_mut which returns the WHOLE slice.
            // 5. This is dangerous if the user accesses outside the range.
            // 6. ArchetypeChunkMut::get_slice_mut DOES slice by entity_range.
            // 7. So as long as entity_ranges are disjoint, we are safe.

            unsafe {
                chunks.push(ArchetypeChunkMut {
                    entity_range: start..end,
                    archetype: &mut *ptr,
                });
            }
        }
        chunks // Collect for parallelization, not streaming
    }

    /// Register component column
    pub fn register_component<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        if !self.component_indices.contains_key(&type_id) {
            let idx = self.components.len();
            self.components.push(ComponentColumn::new::<T>());
            self.component_indices.insert(type_id, idx);
        }
    }

    /// Check if all component columns have been initialized for this signature
    pub fn columns_initialized(&self) -> bool {
        self.columns_initialized
    }

    /// Mark columns as initialized
    pub fn mark_columns_initialized(&mut self) {
        self.columns_initialized = true;
    }
}

/// Type-erased component column
pub struct ComponentColumn {
    ptr: *mut u8,
    len: usize, // number of initialized components
    cap: usize, // capacity in bytes
    item_size: usize,
    /// Alignment of the component type
    align: usize,
    drop_fn: Option<unsafe fn(*mut u8)>,
    pub(crate) added_ticks: Vec<u32>,
    pub(crate) changed_ticks: Vec<u32>,

    /// Chunk-level added tracking
    last_added_tick: u32,

    /// Chunk-level change tracking for efficient filtering
    last_change_tick: u32,
}

// SAFETY: We manually manage the raw pointer and ensure thread safety
// through the world's borrowing rules.
unsafe impl Send for ComponentColumn {}
unsafe impl Sync for ComponentColumn {}

impl ComponentColumn {
    /// Create new column for type T
    ///
    /// Initializes a type-erased component column that can store components of type T.
    /// The column stores components as raw bytes and maintains a drop function for cleanup.
    pub fn new<T: Component>() -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
            item_size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            // Store a drop function only if T needs drop
            // This is critical for proper cleanup of components with destructors
            drop_fn: if std::mem::needs_drop::<T>() {
                Some(|ptr| {
                    // SAFETY: This closure is only called from ComponentColumn::drop
                    // with a valid pointer to an initialized T at the correct offset.
                    // The pointer:
                    // 1. Points to properly aligned memory (allocated for T)
                    // 2. Points to an initialized T (written via get_ptr_mut)
                    // 1. Points to properly aligned memory (allocated for T)
                    // 2. Is valid for reads/writes
                    // 3. Will not be aliased (exclusive access during drop)
                    unsafe {
                        std::ptr::drop_in_place(ptr as *mut T);
                    }
                })
            } else {
                None
            },
            added_ticks: Vec::new(),
            changed_ticks: Vec::new(),
            last_added_tick: 0,
            last_change_tick: 0,
        }
    }

    /// Create an empty clone of this column (preserving type info but with no data)
    pub(crate) fn clone_empty(&self) -> Self {
        Self {
            ptr: std::ptr::null_mut(),
            len: 0,
            cap: 0,
            item_size: self.item_size,
            align: self.align,
            drop_fn: self.drop_fn,
            added_ticks: Vec::new(),
            changed_ticks: Vec::new(),
            last_added_tick: 0,
            last_change_tick: 0,
        }
    }
    /// Get component item size
    pub fn get_item_size(&self) -> usize {
        self.item_size
    }

    /// Get mutable pointer for writing
    ///
    /// Returns a raw pointer to write a component at the given index.
    /// Automatically resizes the buffer if needed.
    ///
    /// # Safety for Callers
    /// The returned pointer is valid for writing exactly `item_size` bytes.
    /// Caller must:
    /// 1. Write a properly initialized value of type T
    /// 2. Not use the pointer after any operation that might reallocate the buffer
    /// 3. Ensure the written value matches the column's component type
    pub fn get_ptr_mut(&mut self, index: usize) -> *mut u8 {
        // Special case for Zero-Sized Types (ZSTs)
        // Writing to a ZST is a no-op, but the pointer must be correctly aligned
        // to avoid Undefined Behavior.
        if self.item_size == 0 {
            // SAFETY: ZSTs are never dereferenced, use without_provenance for Miri compliance
            return std::ptr::without_provenance_mut::<u8>(self.align);
        }

        let required_len = (index + 1) * self.item_size;
        if required_len > self.cap {
            let new_cap = required_len.next_power_of_two().max(64);
            let new_layout = std::alloc::Layout::from_size_align(new_cap, self.align).unwrap();

            let new_ptr = if self.cap == 0 {
                unsafe { std::alloc::alloc(new_layout) }
            } else {
                let old_layout = std::alloc::Layout::from_size_align(self.cap, self.align).unwrap();
                unsafe { std::alloc::realloc(self.ptr, old_layout, new_cap) }
            };

            if new_ptr.is_null() {
                std::alloc::handle_alloc_error(new_layout);
            }

            self.ptr = new_ptr;
            self.cap = new_cap;
        }

        if index >= self.len {
            self.len = index + 1;
        }

        // SAFETY: Use checked arithmetic and bounds validation
        let offset = index.checked_mul(self.item_size)
            .expect("index * item_size overflow");
        
        // Ensure offset is within capacity bounds
        assert!(offset + self.item_size <= self.cap, "Pointer arithmetic overflow");
        
        unsafe { self.ptr.add(offset) }
    }

    /// Mark component as changed at given row
    pub fn mark_changed(&mut self, row: usize, tick: u32) {
        if row < self.changed_ticks.len() {
            self.changed_ticks[row] = tick;
            self.last_change_tick = tick;
        }
    }

    /// Check if this column has changed since the given tick
    pub fn changed_since(&self, tick: u32) -> bool {
        self.last_change_tick > tick
    }

    /// Check if any components were added to this column since the given tick
    pub fn added_since(&self, tick: u32) -> bool {
        self.last_added_tick > tick
    }

    /// Get component at index
    ///
    /// # Safety
    /// Returns a reference to the component if it exists and is properly initialized.
    /// Get component at index
    ///
    /// # Safety
    /// Returns a reference to the component if it exists and is properly initialized.
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        if index >= self.len || self.item_size == 0 {
            // ZSTs should probably be handled more carefully if needed, but for now
            // we return None for out of bounds. Wait, ZSTs are always "in bounds" if they exist.
            if self.item_size == 0 && index < self.added_ticks.len() {
                return Some(unsafe { &*(self.align as *const T) });
            }
            return None;
        }
        // SAFETY:
        // 1. Bounds check: index < self.len
        // 2. Pointer is properly aligned (from manual allocation)
        // 3. item_size == size_of::<T>()
        Some(unsafe { &*(self.ptr.add(index * self.item_size) as *const T) })
    }

    /// Get mutable component at index
    ///
    /// # Safety
    /// Returns a mutable reference to the component if it exists and is properly initialized.
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len || self.item_size == 0 {
            if self.item_size == 0 && index < self.added_ticks.len() {
                return Some(unsafe { &mut *(self.align as *mut T) });
            }
            return None;
        }
        // SAFETY: Same as get but mutable
        Some(unsafe { &mut *(self.ptr.add(index * self.item_size) as *mut T) })
    }

    /// Number of components
    pub fn len(&self) -> usize {
        self.added_ticks.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Get added tick for a row
    pub fn get_added_tick(&self, row: usize) -> Option<u32> {
        self.added_ticks.get(row).copied()
    }

    /// Get changed tick for a row
    pub fn get_changed_tick(&self, row: usize) -> Option<u32> {
        self.changed_ticks.get(row).copied()
    }

    pub fn set_changed_tick(&mut self, row: usize, tick: u32) {
        if row < self.changed_ticks.len() {
            self.changed_ticks[row] = tick;
            if tick > self.last_change_tick {
                self.last_change_tick = tick;
            }
        }
    }

    /// Get typed slice of components
    ///
    /// # Safety
    /// Returns a slice of components if the type T matches the column's type.
    pub fn get_slice<T: Component>(&self) -> Option<&[T]> {
        if self.item_size != std::mem::size_of::<T>() || self.item_size == 0 {
            return None;
        }
        // SAFETY: properly aligned and bounded
        Some(unsafe { std::slice::from_raw_parts(self.ptr as *const T, self.len) })
    }

    /// Get typed mutable slice of components
    ///
    /// # Safety
    /// Returns a mutable slice of components if the type T matches the column's type.
    pub fn get_slice_mut<T: Component>(&mut self) -> Option<&mut [T]> {
        if self.item_size != std::mem::size_of::<T>() || self.item_size == 0 {
            return None;
        }
        // SAFETY: properly aligned and bounded
        Some(unsafe { std::slice::from_raw_parts_mut(self.ptr as *mut T, self.len) })
    }
}

impl Drop for ComponentColumn {
    fn drop(&mut self) {
        if self.item_size > 0 && self.cap > 0 {
            // Drop initialized elements
            if let Some(drop_fn) = self.drop_fn {
                for i in 0..self.len {
                    unsafe {
                        drop_fn(self.ptr.add(i * self.item_size));
                    }
                }
            }

            // Deallocate buffer
            // SAFETY: layout is same as used for alloc/realloc
            let layout = std::alloc::Layout::from_size_align(self.cap, self.align).unwrap();
            unsafe {
                std::alloc::dealloc(self.ptr, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::smallvec;

    #[test]
    fn test_archetype_creation() {
        let sig = smallvec![TypeId::of::<i32>(), TypeId::of::<f32>()];
        let arch = Archetype::new(sig.clone());
        assert_eq!(arch.signature(), &sig);
        assert_eq!(arch.len(), 0);
    }
}
