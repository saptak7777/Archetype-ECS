//! Serialization Example
//! 
//! This example demonstrates the serialization capabilities of the ECS:
//! - JSON serialization and deserialization
//! - Binary serialization
//! - File save/load operations
//! - Debug information export

use archetype_ecs::World;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PlayerData {
    name: String,
    score: u32,
    level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Health {
    current: f32,
    max: f32,
}

fn main() {
    println!("=== Serialization Example ===\n");
    
    // Create world and spawn entities
    let mut world = World::new();
    
    println!("Creating entities with serializable components...");
    
    // Spawn player entities
    for i in 0..5 {
        world.spawn((
            PlayerData {
                name: format!("Player_{}", i + 1),
                score: i * 100,
                level: (i + 1) as u8,
            },
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
    
    // Spawn some NPC entities
    for i in 0..10 {
        world.spawn((
            Position {
                x: (i + 10) as f32,
                y: 0.0,
                z: (i + 10) as f32,
            },
            Health {
                current: 50.0,
                max: 50.0,
            },
        ));
    }
    
    println!("Created {} entities\n", world.entity_count());
    
    // Demonstrate JSON serialization
    println!("=== JSON Serialization ===");
    match world.serialize_json() {
        Ok(json) => {
            println!("Successfully serialized to JSON ({} bytes)", json.len());
            println!("JSON preview: {}", &json[..json.len().min(100)]);
            if json.len() > 100 {
                println!("...");
            }
            
            // Test deserialization
            match World::deserialize_json(&json) {
                Ok(restored_world) => {
                    println!("✅ Successfully deserialized from JSON");
                    println!("Restored world tick: {}", restored_world.tick);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize from JSON: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to serialize to JSON: {e:?}");
        }
    }
    
    // Demonstrate pretty JSON
    println!("\n=== Pretty JSON ===");
    match world.serialize_json_pretty() {
        Ok(pretty_json) => {
            println!("Successfully serialized to pretty JSON");
            println!("Pretty JSON preview:\n{}", &pretty_json[..pretty_json.len().min(200)]);
            if pretty_json.len() > 200 {
                println!("...\n[truncated]");
            }
        }
        Err(e) => {
            println!("❌ Failed to serialize to pretty JSON: {e:?}");
        }
    }
    
    // Demonstrate binary serialization
    println!("\n=== Binary Serialization ===");
    match world.serialize_binary() {
        Ok(binary_data) => {
            println!("Successfully serialized to binary ({} bytes)", binary_data.len());
            
            // Test binary deserialization
            match World::deserialize_binary(&binary_data) {
                Ok(restored_world) => {
                    println!("✅ Successfully deserialized from binary");
                    println!("Restored world tick: {}", restored_world.tick);
                }
                Err(e) => {
                    println!("❌ Failed to deserialize from binary: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to serialize to binary: {e:?}");
        }
    }
    
    // Demonstrate file save/load
    println!("\n=== File Save/Load ===");
    match world.save_to_file("world_save.bin") {
        Ok(_) => {
            println!("✅ Successfully saved world to file");
            
            match World::load_from_file("world_save.bin") {
                Ok(loaded_world) => {
                    println!("✅ Successfully loaded world from file");
                    println!("Loaded world tick: {}", loaded_world.tick);
                }
                Err(e) => {
                    println!("❌ Failed to load world from file: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to save world to file: {e:?}");
        }
    }
    
    // Demonstrate debug info export
    println!("\n=== Debug Information Export ===");
    match world.export_debug_info() {
        Ok(debug_info) => {
            println!("✅ Successfully exported debug information:");
            println!("{debug_info}");
        }
        Err(e) => {
            println!("❌ Failed to export debug info: {e:?}");
        }
    }
    
    // Demonstrate component serialization
    println!("\n=== Component Serialization ===");
    let player_data = PlayerData {
        name: "TestPlayer".to_string(),
        score: 1500,
        level: 5,
    };
    
    // Use bincode directly for component serialization
    match bincode::serialize(&player_data) {
        Ok(data) => {
            println!("✅ Serialized component ({} bytes)", data.len());
            
            match bincode::deserialize::<PlayerData>(&data) {
                Ok(restored_component) => {
                    println!("✅ Deserialized component: {restored_component:?}");
                }
                Err(e) => {
                    println!("❌ Failed to deserialize component: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to serialize component: {e:?}");
        }
    }
    
    println!("\n=== Serialization Example Complete ===");
    println!("Note: Full entity serialization would require a component registry");
    println!("for complete type introspection and restoration.");
}
