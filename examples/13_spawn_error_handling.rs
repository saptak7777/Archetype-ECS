//! Example 13: Spawn Error Handling with Detailed Context
//!
//! This example demonstrates the enhanced spawn error handling:
//! - Detailed error messages for spawn failures
//! - Error context for debugging
//! - Graceful error handling in production code

use archetype_ecs::World;

fn main() {
    println!("=== Spawn Error Handling Example ===\n");

    // Create world
    let mut world = World::new();

    println!("1. Normal spawn (using spawn() - panics on error):");
    let entity1 = world.spawn_entity((1.0f32, 2.0f32)); // Position tuple
    println!("   ✅ Spawned entity: {:?}", entity1);

    println!("\n2. Safe spawn (using try_spawn() - returns Result):");
    match world.try_spawn_entity((3.0f32, 100.0f32)) {
        // Position + Health tuple
        Ok(entity2) => println!("   ✅ Spawned entity: {:?}", entity2),
        Err(e) => println!("   ❌ Spawn failed: {}", e),
    }

    println!("\n3. Batch spawn with error handling:");
    let bundles = vec![
        (5.0f32, 0.2f32),  // Position + Velocity
        (7.0f32, 80.0f32), // Position + Health (using f32 for consistency)
        (9.0f32, 0.4f32),  // Position + Velocity
    ];

    match world.spawn_batch(bundles) {
        Ok(entities) => println!("   ✅ Spawned {} entities: {:?}", entities.len(), entities),
        Err(e) => println!("   ❌ Batch spawn failed: {}", e),
    }

    println!("\n4. Error context demonstration:");
    demonstrate_error_context();

    println!("\n5. Production-ready error handling pattern:");
    production_error_handling_example();

    println!("\n=== Key Benefits of Enhanced Error Handling ===");
    println!("✅ Better debugging: Clear error messages with context");
    println!("✅ Graceful failure: Applications can handle errors instead of crashing");
    println!("✅ Production logging: Detailed error information for monitoring");
    println!("✅ User experience: Meaningful error messages instead of generic panics");
    println!("✅ Development speed: Faster debugging with specific error information");

    println!("\n=== Example Complete ===");
}

fn demonstrate_error_context() {
    println!("   In a real application, spawn errors might include:");
    println!("   - EntityCapacityExhausted: 'Entity capacity exhausted: attempted to spawn 1000001, max is 1000000'");
    println!("   - ComponentRegistrationFailed: 'Failed to register component: Type registration failed'");
    println!("   - ArchetypeCreationFailed: 'Failed to create archetype for 5 components: Memory allocation failed'");
    println!("   Each error provides specific context for debugging and monitoring.");
}

fn production_error_handling_example() {
    println!("   ```rust");
    println!("   // Production-ready spawn with error handling");
    println!("   match world.try_spawn((1.0f32, 2.0f32)) {{");
    println!("       Ok(entity) => {{");
    println!("           // Success: proceed with entity");
    println!("           game_state.add_entity(entity);");
    println!("       }}");
    println!("       Err(e) => {{");
    println!("           // Log error for monitoring");
    println!("           error!(\"Failed to spawn entity: {{}}\", e);");
    println!("           ");
    println!("           // Graceful degradation");
    println!("           if e.is_entity_capacity_exhausted() {{");
    println!("               game_state.trigger_gc();");
    println!("               retry_spawn_later();");
    println!("           }} else {{");
    println!("               game_state.notify_error_to_user(e);");
    println!("           }}");
    println!("       }}");
    println!("   }}");
    println!("   ```");
}
