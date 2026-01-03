//! Serialization Example
//!
//! This example demonstrates the new Scene-based serialization system:
//! - Component registration with SerializationRegistry
//! - Scene creation and JSON export
//! - Entity and component serialization

use archetype_ecs::prelude::*;
use archetype_ecs::serialization::{save_world, Scene, SerializationRegistry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

// Implement Reflect for Position
impl archetype_ecs::reflection::Reflect for Position {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn apply(&mut self, value: &dyn archetype_ecs::reflection::Reflect) {
        if let Some(v) = value.as_any().downcast_ref::<Position>() {
            *self = v.clone();
        }
    }

    fn reflect_clone(&self) -> Box<dyn archetype_ecs::reflection::Reflect> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Health {
    current: f32,
    max: f32,
}

impl archetype_ecs::reflection::Reflect for Health {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn apply(&mut self, value: &dyn archetype_ecs::reflection::Reflect) {
        if let Some(v) = value.as_any().downcast_ref::<Health>() {
            *self = v.clone();
        }
    }

    fn reflect_clone(&self) -> Box<dyn archetype_ecs::reflection::Reflect> {
        Box::new(self.clone())
    }
}

fn main() {
    println!("=== Archetype ECS Serialization Example ===\n");

    // Create world and spawn entities
    let mut world = World::new();

    println!("Creating entities...");
    for i in 0..5 {
        world.spawn_entity((
            Position {
                x: i as f32 * 10.0,
                y: 0.0,
                z: i as f32 * 5.0,
            },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));
    }

    println!("Created {} entities\n", world.entity_count());

    // Create serialization registry and register component types
    println!("=== Setting up Serialization Registry ===");
    let mut registry = SerializationRegistry::new();
    registry.register::<Position>();
    registry.register::<Health>();
    println!("✅ Registered Position and Health components\n");

    // Save world to scene
    println!("=== Saving World to Scene ===");
    match save_world(&world, &registry) {
        Ok(scene) => {
            println!("✅ Successfully created scene");
            println!("Scene contains {} entities", scene.entity_count());

            // Serialize scene to JSON
            match serde_json::to_string_pretty(&scene) {
                Ok(json) => {
                    println!("\n=== Scene JSON ===");
                    println!("{}", json);

                    // Save to file
                    if let Err(e) = std::fs::write("scene.json", &json) {
                        println!("\n❌ Failed to write scene to file: {}", e);
                    } else {
                        println!("\n✅ Saved scene to scene.json");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to serialize scene to JSON: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to save world: {:?}", e);
        }
    }

    println!("\n=== Serialization Example Complete ===");
    println!("\nNote: Full entity deserialization requires additional infrastructure");
    println!("for spawning entities from Box<dyn Reflect> components.");
    println!("This is planned for future iterations.");
}
