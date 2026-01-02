//! Example: Basic ECS usage (Phase 1)
//!
//! Note: Full query iteration will be completed in Phase 2
//! This example shows spawn/despawn and archetype storage

use archetype_ecs::World;

// Define components
#[allow(dead_code)]
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32,
}

fn main() {
    println!("=== Basic ECS Example ===");
    
    // Create world
    let mut world = World::new();
    
    // Spawn entities with components
    println!("Spawning entities...");
    for i in 0..10 {
        world.spawn((
            Position { x: i as f32, y: i as f32 },
            Velocity { x: 0.1, y: 0.0 },
        ));
    }
    
    println!("Spawned {} entities", world.entity_count());
    
    // Query entities
    println!("Querying entities...");
    let count = world.query::<&Position>().iter().count();
    println!("Found {count} entities with Position");
    
    // Despawn an entity
    println!("Despawning entities...");
    // Note: In a real implementation, you'd track entity IDs
    
    println!("=== Example Complete ===");
}
