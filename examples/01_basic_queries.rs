//! Example 1: Basic Query Iteration
//! 
//! This example demonstrates the fundamental ECS concepts:
//! - Creating a World
//! - Spawning entities with components
//! - Querying and iterating over components
//! - Basic component patterns

use archetype_ecs::World;

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

fn main() {
    // Create a new world
    let mut world = World::new();
    
    println!("=== Basic Query Iteration Example ===\n");
    
    // Spawn entities with Position and Velocity components
    println!("Spawning 1000 entities with Position and Velocity...");
    for i in 0..1000 {
        world.spawn_entity((
            Position { 
                x: i as f32, 
                y: 0.0, 
                z: 0.0 
            },
            Velocity { 
                x: 0.1, 
                y: 0.0, 
                z: 0.0 
            },
        ));
    }
    
    println!("Spawned {} entities\n", world.entity_count());
    
    // Query entities with both Position and Velocity
    println!("Querying entities with Position and Velocity:");
    let mut count = 0;
    for (pos, vel) in world.query::<(&Position, &Velocity)>().iter() {
        count += 1;
        // Update position based on velocity
        let new_x = pos.x + vel.x;
        
        // Only print first 5 entities to avoid spam
        if count <= 5 {
            println!("  Entity {}: Position({:.1}, {:.1}, {:.1}) + Velocity({:.1}, {:.1}, {:.1}) -> New X: {:.1}", 
                count, pos.x, pos.y, pos.z, vel.x, vel.y, vel.z, new_x);
        }
    }
    
    println!("  ... and {} more entities\n", count - 5);
    
    // Query entities with only Position
    println!("Querying entities with Position only:");
    for pos in world.query::<&Position>().iter().take(3) {
        println!("  Position: ({:.1}, {:.1}, {:.1})", pos.x, pos.y, pos.z);
    }
    
    println!("\n=== Example Complete ===");
}
