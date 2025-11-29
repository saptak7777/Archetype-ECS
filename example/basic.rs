//! Example: Basic ECS usage (Phase 1)
//!
//! Note: Full query iteration will be completed in Phase 2
//! This example shows spawn/despawn and archetype storage

use aaa_ecs::World;

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

#[allow(dead_code)]
#[derive(Debug)]
struct Health(u32);

fn main() {
    let mut world = World::new();

    println!("Creating entities...");

    // Spawn entity with Position + Velocity
    let entity1 = world
        .spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.5 }))
        .expect("spawn entity1");

    println!("Spawned entity {:?}", entity1);

    // Spawn entity with all three components
    let entity2 = world
        .spawn((
            Position { x: 10.0, y: 20.0 },
            Velocity { x: -1.0, y: 2.0 },
            Health(100),
        ))
        .expect("spawn entity2");

    println!("Spawned entity {:?}", entity2);

    // Spawn entity with only Position
    let entity3 = world
        .spawn((Position { x: 5.0, y: 5.0 },))
        .expect("spawn entity3");

    println!("Spawned entity {:?}", entity3);

    // Check entity locations
    if let Some(loc) = world.get_entity_location(entity1) {
        println!(
            "Entity {:?} is in archetype {}, row {}",
            entity1, loc.archetype_id, loc.archetype_row
        );
    }

    // Despawn entity
    world.despawn(entity2).expect("despawn entity2");
    println!("Despawned entity {:?}", entity2);

    // Try to get despawned entity (should fail)
    if world.get_entity_location(entity2).is_none() {
        println!("Entity {:?} no longer exists", entity2);
    }

    println!("\nArchetype summary:");
    for (i, archetype) in world.archetypes().iter().enumerate() {
        println!(
            "  Archetype {}: {} entities, {} component types",
            i,
            archetype.len(),
            archetype.signature().len()
        );
    }

    println!("\nPhase 1 complete! Phase 2 will add query iteration.");
}
