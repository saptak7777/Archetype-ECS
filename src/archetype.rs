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

use crate::component::Component;
use crate::entity::EntityId;

/// Component signature  
pub type ArchetypeSignature = Vec<TypeId>;

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
        Self {
            signature,
            entities: Vec::new(),
            components: Vec::new(),
            component_indices: FxHashMap::default(),
            columns_initialized: false,
        }
    }

    /// Get signature
    pub fn signature(&self) -> &ArchetypeSignature {
        &self.signature
    }

    /// Allocate row for entity
    pub fn allocate_row(&mut self, entity: EntityId) -> usize {
        let row = self.entities.len();
        self.entities.push(entity);
        row
    }

    /// Remove row and return entity that was swapped in
    ///
    /// # Safety
    /// Caller must ensure `row` is a valid index within this archetype.
    /// Returns Some(entity) if another entity was swapped into this row.
    pub unsafe fn remove_row(&mut self, row: usize) -> Option<EntityId> {
        if row >= self.entities.len() {
            return None;
        }

        self.entities.swap_remove(row);

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

    /// Is empty
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
}

impl ComponentColumn {
    /// Create new column for type T
    pub fn new<T: Component>() -> Self {
        Self {
            data: Vec::new(),
            item_size: std::mem::size_of::<T>(),
            drop_fn: if std::mem::needs_drop::<T>() {
                Some(|ptr| unsafe {
                    std::ptr::drop_in_place(ptr as *mut T);
                })
            } else {
                None
            },
        }
    }

    /// Get mutable pointer for writing
    pub fn get_ptr_mut(&mut self, index: usize) -> *mut u8 {
        let offset = index * self.item_size;
        if offset + self.item_size > self.data.len() {
            self.data.resize(offset + self.item_size, 0);
        }
        unsafe { self.data.as_mut_ptr().add(offset) }
    }

    /// Get component at index
    pub fn get<T: Component>(&self, index: usize) -> Option<&T> {
        let offset = index * self.item_size;
        if offset + self.item_size > self.data.len() {
            return None;
        }
        Some(unsafe { &*(self.data.as_ptr().add(offset) as *const T) })
    }

    /// Get mutable component at index
    pub fn get_mut<T: Component>(&mut self, index: usize) -> Option<&mut T> {
        let offset = index * self.item_size;
        if offset + self.item_size > self.data.len() {
            return None;
        }
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
}

impl Drop for ComponentColumn {
    fn drop(&mut self) {
        if let Some(drop_fn) = self.drop_fn {
            let count = self.len();
            for i in 0..count {
                let offset = i * self.item_size;
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

    #[test]
    fn test_archetype_creation() {
        let sig = vec![TypeId::of::<i32>(), TypeId::of::<f32>()];
        let arch = Archetype::new(sig.clone());
        assert_eq!(arch.signature(), &sig);
        assert_eq!(arch.len(), 0);
    }
}
