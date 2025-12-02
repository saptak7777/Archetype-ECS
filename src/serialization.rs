use crate::entity::EntityId;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use slotmap::Key;
use std::collections::HashMap;

/// Entity ID serialization data
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityIdData {
    pub index: u32,
    pub generation: u32,
}

impl EntityIdData {
    pub fn from_entity_id(id: EntityId) -> Self {
        // Extract the raw data from slotmap KeyData
        let raw = id.data().as_ffi();
        Self {
            index: (raw & 0xFFFFFFFF) as u32,
            generation: ((raw >> 32) & 0xFFFFFFFF) as u32,
        }
    }

    pub fn to_entity_id(&self) -> EntityId {
        // Reconstruct KeyData from index and generation
        let raw = ((self.generation as u64) << 32) | (self.index as u64);
        EntityId::from(slotmap::KeyData::from_ffi(raw))
    }
}

/// Entity data for serialization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityData {
    pub id: EntityIdData,
    pub components: HashMap<String, serde_json::Value>,
}

/// Complete world serialization data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldData {
    /// Format version for migrations
    pub version: u32,
    /// When this save was created
    pub timestamp: u64,
    /// All entities
    pub entities: Vec<EntityData>,
    /// Optional: game metadata
    pub metadata: HashMap<String, String>,
}

impl WorldData {
    /// Create new world data
    pub fn new() -> Self {
        Self {
            version: 1,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            entities: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Serialize to JSON string
    pub fn to_json_string(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("JSON serialization failed: {e}"))
        })
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec_pretty(self).map_err(|e| {
            crate::error::EcsError::SerializationError(format!("JSON serialization failed: {e}"))
        })
    }

    /// Serialize to binary (using bincode)
    pub fn to_binary_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| {
            crate::error::EcsError::SerializationError(format!(
                "Binary serialization failed: {e}"
            ))
        })
    }

    /// Deserialize from JSON string
    pub fn from_json_string(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| {
            crate::error::EcsError::DeserializationError(format!(
                "JSON deserialization failed: {e}"
            ))
        })
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| {
            crate::error::EcsError::DeserializationError(format!(
                "JSON deserialization failed: {e}"
            ))
        })
    }

    /// Deserialize from binary
    pub fn from_binary_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| {
            crate::error::EcsError::DeserializationError(format!(
                "Binary deserialization failed: {e}"
            ))
        })
    }

    /// Get number of entities
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Add entity data
    pub fn add_entity(&mut self, entity: EntityData) {
        self.entities.push(entity);
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

impl Default for WorldData {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_data_creation() {
        let world = WorldData::new();
        assert_eq!(world.version, 1);
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_json_serialization() {
        let mut world = WorldData::new();
        world.add_metadata("level".to_string(), "1".to_string());

        let json = world.to_json_string().unwrap();
        assert!(json.contains("\"version\":1") || json.contains("\"version\": 1"));

        let world2 = WorldData::from_json_string(&json).unwrap();
        assert_eq!(world2.version, 1);
    }

    #[test]
    fn test_binary_serialization() {
        let mut world = WorldData::new();
        world.add_metadata("test".to_string(), "data".to_string());

        let bytes = world.to_binary_bytes().unwrap();
        let world2 = WorldData::from_binary_bytes(&bytes).unwrap();

        assert_eq!(world2.version, 1);
        assert_eq!(world2.metadata.get("test"), Some(&"data".to_string()));
    }
}
