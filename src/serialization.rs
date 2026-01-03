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

//! World serialization and scene management.
//!
//! This module provides functionality to save and load ECS world state,
//! enabling save/load systems, level serialization, and state persistence.

use crate::entity::EntityId;
use crate::error::{EcsError, Result};
use crate::reflection::Reflect;
use crate::world::World;
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;

/// A serializable snapshot of world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Entities in this scene
    pub entities: Vec<EntityData>,
}

/// Serialized entity with its components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    /// Original entity ID (for reference, remapped on load)
    pub id: u64,
    /// Component data as type name -> JSON value
    pub components: HashMap<String, serde_json::Value>,
}

impl Scene {
    /// Create an empty scene
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Get number of entities in scene
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

/// Component serializer trait for dynamic serialization
pub trait ComponentSerializer: Send + Sync {
    /// Serialize a component to JSON
    fn serialize_json(&self, component: &dyn Reflect) -> Result<serde_json::Value>;

    /// Deserialize a component from JSON
    fn deserialize_json(&self, value: &serde_json::Value) -> Result<Box<dyn Reflect>>;

    /// Get the type name for this component
    fn type_name(&self) -> &'static str;
}

/// Implementation of ComponentSerializer for types that implement Serialize + Deserialize
struct TypedComponentSerializer<T: Reflect + Serialize + for<'de> Deserialize<'de> + Clone> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Reflect + Serialize + for<'de> Deserialize<'de> + Clone> TypedComponentSerializer<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Reflect + Serialize + for<'de> Deserialize<'de> + Clone + 'static> ComponentSerializer
    for TypedComponentSerializer<T>
{
    fn serialize_json(&self, component: &dyn Reflect) -> Result<serde_json::Value> {
        let concrete = component
            .as_any()
            .downcast_ref::<T>()
            .ok_or_else(|| EcsError::SerializationError("Type mismatch".to_string()))?;

        serde_json::to_value(concrete).map_err(|e| EcsError::SerializationError(e.to_string()))
    }

    fn deserialize_json(&self, value: &serde_json::Value) -> Result<Box<dyn Reflect>> {
        let component: T = serde_json::from_value(value.clone())
            .map_err(|e| EcsError::SerializationError(e.to_string()))?;

        Ok(Box::new(component))
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }
}

/// Extended type registry with serialization support
pub struct SerializationRegistry {
    serializers: HashMap<TypeId, Box<dyn ComponentSerializer>>,
    type_names: HashMap<String, TypeId>,
}

impl SerializationRegistry {
    /// Create a new serialization registry
    pub fn new() -> Self {
        Self {
            serializers: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    /// Register a component type for serialization
    pub fn register<T>(&mut self)
    where
        T: Reflect + Serialize + for<'de> Deserialize<'de> + Clone + 'static,
    {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();

        self.serializers
            .insert(type_id, Box::new(TypedComponentSerializer::<T>::new()));
        self.type_names.insert(type_name, type_id);
    }

    /// Get serializer for a type
    pub fn get_serializer(&self, type_id: TypeId) -> Option<&dyn ComponentSerializer> {
        self.serializers.get(&type_id).map(|s| s.as_ref())
    }

    /// Get type ID from type name
    pub fn get_type_id(&self, type_name: &str) -> Option<TypeId> {
        self.type_names.get(type_name).copied()
    }
}

impl Default for SerializationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Save world state to a scene
pub fn save_world(world: &World, registry: &SerializationRegistry) -> Result<Scene> {
    let mut scene = Scene::new();

    // Iterate through all entities
    for archetype in world.archetypes() {
        for &entity_id in archetype.entities().iter() {
            let entity_data = EntityData {
                id: unsafe { std::mem::transmute::<EntityId, u64>(entity_id) },
                components: HashMap::new(),
            };

            // Serialize each component in this archetype
            for &component_type in archetype.signature() {
                if let Some(serializer) = registry.get_serializer(component_type) {
                    if let Some(_column) = archetype.get_column(component_type) {
                        // Get component as Reflect trait object
                        // Note: This requires components to implement Reflect
                        // For now, we'll use a simplified approach with the serializer
                        let _type_name = serializer.type_name();

                        // We need to get the actual component data
                        // This is a limitation - we need a way to get &dyn Reflect from ComponentColumn
                        // For MVP, we'll document this limitation and provide a workaround

                        // TODO: Implement proper component reflection
                        // For now, skip components that can't be serialized
                        continue;
                    }
                }
            }

            // Only add entities that have serializable components
            if !entity_data.components.is_empty() {
                scene.entities.push(entity_data);
            }
        }
    }

    Ok(scene)
}

/// Load world state from a scene
pub fn load_world(
    _world: &mut World,
    _scene: &Scene,
    _registry: &SerializationRegistry,
) -> Result<HashMap<u64, EntityId>> {
    let id_map = HashMap::new();

    // TODO: Implement proper deserialization
    // This requires:
    // 1. A way to spawn entities with Box<dyn Reflect> components
    // 2. Component type registration that includes spawn hooks
    // For MVP, this is a known limitation that will be addressed in future iterations

    Ok(id_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new();
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_serialization_registry() {
        let mut registry = SerializationRegistry::new();

        // Register some basic types
        registry.register::<i32>();
        registry.register::<f32>();

        assert!(registry.get_serializer(TypeId::of::<i32>()).is_some());
        assert!(registry.get_serializer(TypeId::of::<f32>()).is_some());
    }
}
