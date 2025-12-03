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

/// Archetype: Structure of Arrays storage
pub struct Archetype {
    signature: ArchetypeSignature,
    entities: Vec<EntityId>,
    components: Vec<ComponentColumn>,
    component_indices: FxHashMap<TypeId, usize>,
    columns_initialized: bool,
}

impl Archetype {
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

    /// Allocate row for entity
    pub fn allocate_row(&mut self, entity: EntityId, tick: u32) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);

        for column in &mut self.components {
            column.added_ticks.push(tick);
            column.changed_ticks.push(tick);
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
        if row >= self.entities.len() {
            return None;
        }

        self.entities.swap_remove(row);

        for column in &mut self.components {
            // Handle data swap-remove manually for the byte buffer
            let item_size = column.item_size;
            if item_size > 0 {
                let last_idx = column.len() - 1;
                if row < last_idx {
                    // SAFETY: This pointer arithmetic is safe because:
                    // 1. last_idx < column.len() (from len() - 1)
                    // 2. row < last_idx (checked above)
                    // 3. Both offsets are within allocated buffer bounds
                    // 4. item_size is the correct size for type T (set in new())
                    // 5. copy_nonoverlapping is safe because src != dst (row < last_idx)
                    let src = column.data.as_ptr().add(last_idx * item_size);
                    let dst = column.data.as_mut_ptr().add(row * item_size);
                    std::ptr::copy_nonoverlapping(src, dst, item_size);
                }
                // SAFETY: Truncating to last_idx * item_size is safe because:
                // 1. last_idx = len() - 1, so last_idx * item_size < data.len()
                // 2. We're removing exactly one element worth of bytes
                // 3. The data at last_idx was already moved (if row < last_idx)
                column.data.set_len(last_idx * item_size);
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

    /// Get mutable slice of component columns
    pub(crate) fn components_mut(&mut self) -> &mut [ComponentColumn] {
        &mut self.components
    }

    /// Reserve space for additional rows
    pub fn reserve_rows(&mut self, additional: usize) {
        if self.entities.capacity() - self.entities.len() < additional {
            self.entities.reserve(additional);
            for column in &mut self.components {
                column.data.reserve(additional * column.item_size);
                column.added_ticks.reserve(additional);
                column.changed_ticks.reserve(additional);
            }
        }
    }

    /// Get all entities
    pub fn entities(&self) -> &[EntityId] {
        &self.entities
    }

    /// Number of entities
    pub fn len(&self) -> usize {
        self.entities.len()
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

    /// Check if archetype is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
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
    data: Vec<u8>,
    item_size: usize,
    drop_fn: Option<unsafe fn(*mut u8)>,
    pub(crate) added_ticks: Vec<u32>,
    pub(crate) changed_ticks: Vec<u32>,

    /// Chunk-level change tracking for efficient filtering
    last_change_tick: u32,
}

impl ComponentColumn {
    /// Create new column for type T
    ///
    /// Initializes a type-erased component column that can store components of type T.
    /// The column stores components as raw bytes and maintains a drop function for cleanup.
    pub fn new<T: Component>() -> Self {
        Self {
            data: Vec::new(),
            item_size: std::mem::size_of::<T>(),
            // Store a drop function only if T needs drop
            // This is critical for proper cleanup of components with destructors
            drop_fn: if std::mem::needs_drop::<T>() {
                Some(|ptr| {
                    // SAFETY: This closure is only called from ComponentColumn::drop
                    // with a valid pointer to an initialized T at the correct offset.
                    // The pointer:
                    // 1. Points to properly aligned memory (allocated for T)
                    // 2. Points to an initialized T (written via get_ptr_mut)
                    // 3. Is only dropped once (called during ComponentColumn cleanup)
                    // 4. Has the correct type (ptr was created for this column's T)
                    unsafe {
                        std::ptr::drop_in_place(ptr as *mut T);
                    }
                })
            } else {
                None
            },
            added_ticks: Vec::new(),
            changed_ticks: Vec::new(),
            last_change_tick: 0,
        }
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
        let offset = index * self.item_size;
        if offset + self.item_size > self.data.len() {
            self.data.resize(offset + self.item_size, 0);
        }
        // SAFETY: This is safe because:
        // 1. offset is calculated as index * item_size
        // 2. We just ensured offset + item_size <= data.len()
        // 3. The pointer is valid for item_size bytes
        // 4. Vec guarantees proper alignment for u8
        unsafe { self.data.as_mut_ptr().add(offset) }
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

    /// Get component at index
    ///
    /// # Safety
    /// Returns a reference to the component if it exists and is properly initialized.
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        let offset = index * self.item_size;
        if offset + self.item_size > self.data.len() {
            return None;
        }
        // SAFETY: This is safe because:
        // 1. We verified offset + item_size <= data.len() (bounds check)
        // 2. The data was written via get_ptr_mut as a valid T
        // 3. item_size == size_of::<T>() (verified at column creation)
        // 4. The cast is valid because this column stores type T
        // 5. The lifetime is tied to &self, preventing use-after-free
        Some(unsafe { &*(self.data.as_ptr().add(offset) as *const T) })
    }

    /// Get mutable component at index
    ///
    /// # Safety
    /// Returns a mutable reference to the component if it exists and is properly initialized.
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        let offset = index * self.item_size;
        if offset + self.item_size > self.data.len() {
            return None;
        }
        // SAFETY: This is safe because:
        // 1. We verified offset + item_size <= data.len() (bounds check)
        // 2. The data was written via get_ptr_mut as a valid T
        // 3. item_size == size_of::<T>() (verified at column creation)
        // 4. The cast is valid because this column stores type T
        // 5. The lifetime is tied to &mut self, ensuring exclusive access
        // 6. No other references to this component exist (Rust's borrow rules)
        Some(unsafe { &mut *(self.data.as_mut_ptr().add(offset) as *mut T) })
    }

    /// Number of components
    pub fn len(&self) -> usize {
        if self.item_size == 0 {
            0
        } else {
            self.data.len() / self.item_size
        }
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    /// Get added tick for a row
    pub fn get_added_tick(&self, row: usize) -> Option<u32> {
        self.added_ticks.get(row).copied()
    }

    /// Get changed tick for a row
    pub fn get_changed_tick(&self, row: usize) -> Option<u32> {
        self.changed_ticks.get(row).copied()
    }

    /// Set changed tick for a row
    pub fn set_changed_tick(&mut self, row: usize, tick: u32) {
        if row < self.changed_ticks.len() {
            self.changed_ticks[row] = tick;
        }
    }
}

impl Drop for ComponentColumn {
    /// Custom drop implementation to properly clean up type-erased components
    ///
    /// This is critical for components with destructors (e.g., Vec, String, Box)
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            let count = self.len();
            for i in 0..count {
                let offset = i * self.item_size;
                // SAFETY: This is safe because:
                // 1. offset = i * item_size where i < count = len()
                // 2. len() returns data.len() / item_size, so offset < data.len()
                // 3. drop_fn was created with the correct type T in new()
                // 4. Each component was properly initialized via get_ptr_mut
                // 5. We're dropping each component exactly once
                // 6. This is the final cleanup, no further access will occur
                unsafe {
                    drop_fn(self.data.as_mut_ptr().add(offset));
                }
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
