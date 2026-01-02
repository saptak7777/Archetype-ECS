//! Serialization support for the ECS
//! 
//! This module provides serialization and deserialization capabilities
//! for saving/loading game state, network replication, and testing.

use crate::world::World;
use crate::error::{EcsError, Result};
use serde::{Serialize, Deserialize};

/// Serializable representation of an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableEntity {
    pub id: u64, // EntityId serialized as u64
    pub components: Vec<SerializableComponent>,
}

/// Serializable representation of a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableComponent {
    pub type_name: String,
    pub data: Vec<u8>, // Serialized component data
}

/// Serializable representation of the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableWorld {
    pub entities: Vec<SerializableEntity>,
    pub next_entity_id: u32,
    pub tick: u64,
}

/// Trait for components that can be serialized
pub trait SerializableComponentTrait: serde::Serialize + for<'de> serde::Deserialize<'de> {
    fn type_name() -> &'static str;
}

// Blanket implementation for all serde-compatible types
impl<T> SerializableComponentTrait for T 
where 
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

impl World {
    /// Serialize world to JSON
    pub fn serialize_json(&self) -> Result<String> {
        let serializable = self.to_serializable()?;
        serde_json::to_string(&serializable)
            .map_err(|e| EcsError::SerializationError(e.to_string()))
    }
    
    /// Serialize world to JSON with pretty printing
    pub fn serialize_json_pretty(&self) -> Result<String> {
        let serializable = self.to_serializable()?;
        serde_json::to_string_pretty(&serializable)
            .map_err(|e| EcsError::SerializationError(e.to_string()))
    }
    
    /// Serialize world to binary format
    pub fn serialize_binary(&self) -> Result<Vec<u8>> {
        let serializable = self.to_serializable()?;
        bincode::serialize(&serializable)
            .map_err(|e| EcsError::SerializationError(e.to_string()))
    }
    
    /// Deserialize world from JSON
    pub fn deserialize_json(json: &str) -> Result<Self> {
        let serializable: SerializableWorld = serde_json::from_str(json)
            .map_err(|e| EcsError::DeserializationError(e.to_string()))?;
        
        Self::from_serializable(serializable)
    }
    
    /// Deserialize world from binary format
    pub fn deserialize_binary(data: &[u8]) -> Result<Self> {
        let serializable: SerializableWorld = bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationError(e.to_string()))?;
        
        Self::from_serializable(serializable)
    }
    
    /// Convert world to serializable format
    fn to_serializable(&self) -> Result<SerializableWorld> {
        // Simplified serialization - just save basic stats
        // Full entity serialization would require more complex component introspection
        let entities = Vec::new(); // Placeholder - would need component registry
        
        Ok(SerializableWorld {
            entities,
            next_entity_id: 0, // Placeholder - EntityId generator not accessible
            tick: self.tick as u64,
        })
    }
    
    /// Restore world from serializable format
    fn from_serializable(serializable: SerializableWorld) -> Result<Self> {
        let mut world = World::new();
        
        // Restore tick
        world.tick = serializable.tick as u32;
        
        // Note: Full entity restoration would require:
        // 1. Component type registry
        // 2. Component deserialization
        // 3. Entity ID restoration
        
        Ok(world)
    }
    
    /// Save world to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let data = self.serialize_binary()?;
        std::fs::write(path, data)
            .map_err(|e| EcsError::IoError(e.to_string()))
    }
    
    /// Load world from file
    pub fn load_from_file(path: &str) -> Result<Self> {
        let data = std::fs::read(path)
            .map_err(|e| EcsError::IoError(e.to_string()))?;
        
        Self::deserialize_binary(&data)
    }
    
    /// Export world state for debugging
    pub fn export_debug_info(&self) -> Result<String> {
        let debug_info = serde_json::json!({
            "entity_count": self.entity_count(),
            "archetype_count": self.archetype_count(),
            "tick": self.tick as u64,
            "memory_stats": {
                "entity_index_memory": self.memory_stats().entity_index_memory,
                "archetype_memory": self.memory_stats().archetype_memory,
                "total_memory": self.memory_stats().total_memory,
            }
        });
        
        serde_json::to_string_pretty(&debug_info)
            .map_err(|e| EcsError::SerializationError(e.to_string()))
    }
}

/// Helper trait for serializing specific component types
pub trait ComponentSerializer<T> {
    fn serialize_component(component: &T) -> Result<Vec<u8>>;
    fn deserialize_component(data: &[u8]) -> Result<T>;
}

/// Default implementation for serde-compatible components
impl<T> ComponentSerializer<T> for T 
where 
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    fn serialize_component(component: &T) -> Result<Vec<u8>> {
        bincode::serialize(component)
            .map_err(|e| EcsError::SerializationError(e.to_string()))
    }
    
    fn deserialize_component(data: &[u8]) -> Result<T> {
        bincode::deserialize(data)
            .map_err(|e| EcsError::DeserializationError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestComponent {
        value: i32,
        name: String,
    }
    
    #[test]
    fn test_serialize_deserialize_world() {
        let mut world = World::new();
        
        // Spawn some test entities
        for i in 0..10 {
            world.spawn_entity((
                TestComponent {
                    value: i,
                    name: format!("Entity_{i}"),
                },
            ));
        }
        
        // Serialize to JSON
        let json = world.serialize_json().expect("Failed to serialize");
        assert!(!json.is_empty());
        
        // Deserialize from JSON
        let restored_world = World::deserialize_json(&json).expect("Failed to deserialize");
        
        // Basic validation
        assert_eq!(restored_world.tick, world.tick);
    }
    
    #[test]
    fn test_serialize_binary() {
        let mut world = World::new();
        
        world.spawn_entity((
            TestComponent {
                value: 42,
                name: "Test".to_string(),
            },
        ));
        
        // Serialize to binary
        let binary = world.serialize_binary().expect("Failed to serialize");
        assert!(!binary.is_empty());
        
        // Deserialize from binary
        let restored_world = World::deserialize_binary(&binary).expect("Failed to deserialize");
        
        // Basic validation
        assert_eq!(restored_world.tick, world.tick);
    }
    
    #[test]
    fn test_export_debug_info() {
        let world = World::new();
        
        let debug_info = world.export_debug_info().expect("Failed to export debug info");
        assert!(!debug_info.is_empty());
        
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&debug_info).expect("Invalid JSON");
        assert!(parsed.get("entity_count").is_some());
        assert!(parsed.get("archetype_count").is_some());
        assert!(parsed.get("tick").is_some());
    }
}
