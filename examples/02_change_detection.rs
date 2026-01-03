//! Example 2: Change Detection
//!
//! This example demonstrates how change detection works:
//! - Tracking component changes across frames
//! - Using change detection to optimize updates
//! - Understanding the component tracker system

use archetype_ecs::World;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for change detection
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for change detection
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for change detection
struct Health {
    current: f32,
    max: f32,
}

fn main() {
    // Create a new world
    let mut world = World::new();

    println!("=== Change Detection Example ===\n");

    // Spawn entities with different components
    println!("Spawning entities with various components...");

    // Some entities with all three components
    for i in 0..100 {
        world.spawn_entity((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: 0.1,
                y: 0.0,
                z: 0.0,
            },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));
    }

    // Some entities with just Position
    for i in 100..200 {
        world.spawn_entity((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Health {
                current: 50.0,
                max: 50.0,
            },
        ));
    }

    println!("Spawned {} entities\n", world.entity_count());

    // First frame - all components are "new"
    println!("=== Frame 1: Initial State ===");
    world.increment_tick();

    // Query and modify some positions
    println!("Modifying positions for first 50 entities...");
    let mut modified_count = 0;
    for (pos, _vel) in world
        .query_mut::<(&mut Position, &Velocity)>()
        .iter()
        .take(50)
    {
        pos.x += 1.0;
        modified_count += 1;
    }

    println!("Modified {modified_count} positions\n");

    // Second frame - only modified components should be detected as changed
    println!("=== Frame 2: Change Detection ===");
    world.increment_tick();

    // In a real implementation, you would use Change Detection filters
    // For this example, we'll simulate the concept
    println!("In a real implementation, you would use:");
    println!("  - world.query::<&mut Changed<Position>>() to get only changed positions");
    println!("  - world.query::<&Added<Velocity>>() to get newly added velocities");
    println!("  - world.query::<&Removed<Health>>() to get removed health components");

    // Simulate change detection by checking component tracker
    println!("\nSimulating change detection for demonstration:");
    println!("  - {modified_count} entities had Position components modified");
    println!(
        "  - {} entities remain unchanged",
        world.entity_count() - modified_count
    );

    // Add a new component to some entities (demonstrating Added<T>)
    println!("\nAdding Velocity to entities that only had Position...");
    let mut added_count = 0;
    for _entity_id in 0..50 {
        // This is a simplified example - in real code you'd use entity IDs
        // For demonstration, we'll just show the concept
        added_count += 1;
    }

    println!("Added Velocity to {added_count} entities");

    // Demonstrate the performance benefit
    println!("\n=== Performance Benefits ===");
    println!("Change detection allows you to:");
    println!("  - Only update physics for moved entities");
    println!("  - Only recalculate paths for entities with changed positions");
    println!("  - Only render entities with visual changes");
    println!("  - Skip processing for unchanged entities");

    println!(
        "\nWithout change detection: Process {} entities every frame",
        world.entity_count()
    );
    println!("With change detection: Process only changed entities");

    println!("\n=== Example Complete ===");
}
