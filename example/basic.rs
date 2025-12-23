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

#[allow(dead_code)]
#[derive(Debug)]
struct Health(u32);

fn main() {
    let mut world = World::new();

    println!("Creating entities...");

    // Spawn entity with Position + Velocity
    let entity1 = world.spawn((Position { x: 0.0, y: 0.0 }, Velocity { x: 1.0, y: 0.5 }));

    println!("Spawned entity {entity1:?}");

    // Spawn entity with all three components
    let entity2 = world.spawn((
        Position { x: 10.0, y: 20.0 },
        Velocity { x: -1.0, y: 2.0 },
        Health(100),
    ));

    println!("Spawned entity {entity2:?}");

    // Spawn entity with only Position
    let entity3 = world.spawn((Position { x: 5.0, y: 5.0 },));

    println!("Spawned entity {entity3:?}");

    // Check entity locations
    if let Some(loc) = world.get_entity_location(entity1) {
        println!(
            "Entity {:?} is in archetype {}, row {}",
            entity1, loc.archetype_id, loc.archetype_row
        );
    }

    // Despawn entity
    world.despawn(entity2).expect("despawn entity2");
    println!("Despawned entity {entity2:?}");

    // Try to get despawned entity (should fail)
    if world.get_entity_location(entity2).is_none() {
        println!("Entity {entity2:?} no longer exists");
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

    // Phase 2: Cached Query
    println!("\nPhase 2: Cached Query");
    use archetype_ecs::query::CachedQuery;
    let mut query = CachedQuery::<(&Position, &Velocity)>::new(&world);

    println!("Iterating query...");
    for (pos, vel) in query.iter(&world) {
        println!(
            "  Entity at ({}, {}) with velocity ({}, {})",
            pos.x, pos.y, vel.x, vel.y
        );
    }

    println!("\nPhase 1 & 2 complete!");
}
