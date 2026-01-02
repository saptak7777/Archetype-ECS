//! Example 15: Standardized Method Naming Conventions
//! 
//! This example demonstrates the new standardized naming patterns:
//! - Consistent verb_noun_mutation pattern
//! - Clear and predictable API
//! - Backward compatibility with deprecation warnings

use archetype_ecs::World;

fn main() {
    println!("=== Standardized Method Naming Conventions ===\n");
    
    let mut world = World::new();
    
    println!("1. Creation/Modification Methods:");
    
    // ✅ New standardized naming
    let entity1 = world.spawn_entity((1.0f32, 2.0f32)); // Position tuple
    println!("   ✅ spawn_entity(): {:?}", entity1);
    
    // ✅ Batch operations
    let bundles = vec![
        (3.0f32, 100.0f32), // Position + Health
        (5.0f32, 0.2f32),   // Position + Velocity
    ];
    let entities = world.spawn_batch(bundles).unwrap();
    println!("   ✅ spawn_batch(): {:?}", entities);
    
    // ✅ Component operations
    world.add_component(entity1, 50.0f32).unwrap(); // Add Health
    println!("   ✅ add_component(): Added Health to entity");
    
    // ✅ Error handling with new methods
    match world.try_spawn_entity((7.0f32,)) {
        Ok(entity2) => println!("   ✅ try_spawn_entity(): {:?}", entity2),
        Err(e) => println!("   ❌ try_spawn_entity() failed: {}", e),
    }
    
    println!("\n2. Access Methods:");
    
    // ✅ Component access
    if let Some(position) = world.get_component::<f32>(entity1) {
        println!("   ✅ get_component(): First component is {}", position);
    }
    
    if let Some(health) = world.get_component_mut::<f32>(entity1) {
        *health = 75.0;
        println!("   ✅ get_component_mut(): Updated component to {}", health);
    }
    
    // ✅ Status checks
    println!("   ✅ has_component<f32>(): {}", world.has_component::<f32>(entity1));
    println!("   ✅ is_alive(): {}", world.is_alive(entity1));
    
    println!("\n3. Query/Iteration Methods:");
    
    // ✅ Query operations
    let mut count = 0;
    for (pos, vel) in world.query_mut::<(&mut f32, &f32)>().iter() {
        *pos += *vel;
        count += 1;
    }
    println!("   ✅ query_mut().iter(): Processed {} entities", count);
    
    // ✅ Status methods
    println!("   ✅ entity_count(): {}", world.entity_count());
    println!("   ✅ archetype_count(): {}", world.archetype_count());
    
    println!("\n4. Backward Compatibility:");
    
    // ⚠️ Old methods still work but show deprecation warnings
    let entity3 = world.spawn((9.0f32, 10.0f32));
    println!("   ⚠️ spawn() (deprecated): {:?}", entity3);
    
    println!("\n=== Naming Convention Benefits ===");
    println!("✅ Predictable API: All methods follow verb_noun_mutation pattern");
    println!("✅ Clear Intent: spawn_entity() is clearer than spawn()");
    println!("✅ Consistent Patterns: get_component(), has_component(), add_component()");
    println!("✅ IDE Autocomplete: Easier to discover related methods");
    println!("✅ Professional Quality: Consistent with industry standards");
    
    println!("\n=== Migration Guide ===");
    println!("Old → New:");
    println!("spawn() → spawn_entity()");
    println!("try_spawn() → try_spawn_entity()");
    println!("Other methods already follow the pattern ✅");
    
    println!("\n=== Example Complete ===");
}
