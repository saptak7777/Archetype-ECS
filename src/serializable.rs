use crate::serialization::{EntityData, EntityIdData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position component (serializable)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializablePosition {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Health component (serializable)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableHealth {
    pub hp: f32,
    pub max_hp: f32,
}

/// Inventory component (serializable)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableInventory {
    pub items: Vec<String>,
    pub max_slots: u32,
}

/// Velocity component (serializable)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableVelocity {
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
}

/// Name component (serializable)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializableName {
    pub name: String,
}

/// Example of building entity from components
pub fn build_entity_data(
    id: u32,
    generation: u32,
    components: Vec<(&str, serde_json::Value)>,
) -> EntityData {
    let mut comp_map = HashMap::new();
    for (name, value) in components {
        comp_map.insert(name.to_string(), value);
    }

    EntityData {
        id: EntityIdData {
            index: id,
            generation,
        },
        components: comp_map,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serializable_position() {
        let pos = SerializablePosition {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let json = serde_json::to_value(&pos).unwrap();
        let pos2: SerializablePosition = serde_json::from_value(json).unwrap();
        assert_eq!(pos, pos2);
    }

    #[test]
    fn test_serializable_health() {
        let health = SerializableHealth {
            hp: 50.0,
            max_hp: 100.0,
        };
        let json = serde_json::to_value(&health).unwrap();
        let health2: SerializableHealth = serde_json::from_value(json).unwrap();
        assert_eq!(health, health2);
    }

    #[test]
    fn test_serializable_inventory() {
        let inv = SerializableInventory {
            items: vec!["sword".to_string(), "shield".to_string()],
            max_slots: 10,
        };
        let json = serde_json::to_value(&inv).unwrap();
        let inv2: SerializableInventory = serde_json::from_value(json).unwrap();
        assert_eq!(inv, inv2);
    }

    #[test]
    fn test_build_entity_data() {
        let entity = build_entity_data(
            42,
            1,
            vec![
                (
                    "Position",
                    serde_json::json!({"x": 10.0, "y": 20.0, "z": 0.0}),
                ),
                ("Health", serde_json::json!({"hp": 100.0, "max_hp": 100.0})),
            ],
        );

        assert_eq!(entity.id.index, 42);
        assert_eq!(entity.id.generation, 1);
        assert_eq!(entity.components.len(), 2);
    }
}
