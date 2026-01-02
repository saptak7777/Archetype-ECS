//! Test file to verify debug visualization methods work correctly

use archetype_ecs::World;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for debug visualization
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for debug visualization
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for debug visualization
struct Health {
    current: f32,
    max: f32,
}

fn main() {
    println!("=== Debug Visualization Test ===\n");
    
    // Create world and spawn entities
    let mut world = World::new();
    
    // Spawn entities with different component combinations
    let entity1 = world.spawn_entity((
        Position { x: 1.0, y: 2.0, z: 3.0 },
        Velocity { x: 0.1, y: 0.0, z: 0.0 },
    ));
    
    let entity2 = world.spawn_entity((
        Position { x: 4.0, y: 5.0, z: 6.0 },
        Health { current: 100.0, max: 100.0 },
    ));
    
    let entity3 = world.spawn_entity((
        Position { x: 7.0, y: 8.0, z: 9.0 },
        Velocity { x: 0.2, y: 0.0, z: 0.0 },
        Health { current: 75.0, max: 100.0 },
    ));
    
    println!("Spawned {} entities\n", world.entity_count());
    
    // Test debug_print_entity
    println!("=== Testing debug_print_entity ===");
    world.debug_print_entity(entity1);
    world.debug_print_entity(entity2);
    world.debug_print_entity(entity3);
    
    // Test debug_print_entities_with
    println!("\n=== Testing debug_print_entities_with ===");
    world.debug_print_entities_with::<Position>();
    world.debug_print_entities_with::<Velocity>();
    world.debug_print_entities_with::<Health>();
    
    // Test debug_print_memory_stats
    println!("\n=== Testing debug_print_memory_stats ===");
    world.debug_print_memory_stats();
    
    // Test debug_print_query_cache_stats
    println!("\n=== Testing debug_print_query_cache_stats ===");
    world.debug_print_query_cache_stats();
    
    println!("\n=== Debug Visualization Test Complete ===");
}
