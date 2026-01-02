//! Example 4: Event System and Observers
//! 
//! This example demonstrates:
//! - Publishing events to the global event bus
//! - Creating custom observers that react to events
//! - Event-driven architecture patterns
//! - Observer lifecycle callbacks

use archetype_ecs::{World, Observer, EntityEvent, ObserverRegistry};
use slotmap::Key;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for event system
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone)]
struct Health {
    current: f32,
    max: f32,
}

// Custom event data
#[derive(Debug, Clone)]
#[allow(dead_code)] // Example event for event system
struct DamageEvent {
    amount: f32,
    source: String,
}

// Observer that tracks entity lifecycle events
struct LifecycleObserver {
    spawn_count: usize,
    despawn_count: usize,
    component_add_count: usize,
    component_remove_count: usize,
}

impl LifecycleObserver {
    fn new() -> Self {
        Self {
            spawn_count: 0,
            despawn_count: 0,
            component_add_count: 0,
            component_remove_count: 0,
        }
    }
}

impl Observer for LifecycleObserver {
    fn on_event(&mut self, event: &EntityEvent, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        match event {
            EntityEvent::Spawned(_) => {
                self.spawn_count += 1;
                println!("  📝 Lifecycle: Entity spawned (total: {})", self.spawn_count);
            }
            EntityEvent::Despawned(_) => {
                self.despawn_count += 1;
                println!("  📝 Lifecycle: Entity despawned (total: {})", self.despawn_count);
            }
            EntityEvent::ComponentAdded(_, _type_id) => {
                self.component_add_count += 1;
                println!("  📝 Lifecycle: Component added (total: {})", self.component_add_count);
            }
            EntityEvent::ComponentRemoved(_, _type_id) => {
                self.component_remove_count += 1;
                println!("  📝 Lifecycle: Component removed (total: {})", self.component_remove_count);
            }
            EntityEvent::Custom(name, _, _) => {
                println!("  📝 Lifecycle: Custom event '{}'", name);
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "LifecycleObserver"
    }

    fn on_before_register(&mut self, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  🔧 LifecycleObserver: Setting up before registration...");
        Ok(())
    }

    fn on_registered(&mut self, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  ✅ LifecycleObserver: Registered and ready!");
        Ok(())
    }

    fn on_after_register(&mut self, _world: &mut World, index: usize) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  📍 LifecycleObserver: Stored at index {}", index);
        Ok(())
    }
}

// Observer that responds to health changes
struct HealthObserver {
    damage_total: f32,
    heal_total: f32,
}

impl HealthObserver {
    fn new() -> Self {
        Self {
            damage_total: 0.0,
            heal_total: 0.0,
        }
    }
}

impl Observer for HealthObserver {
    fn on_event(&mut self, event: &EntityEvent, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        match event {
            EntityEvent::Custom(name, entity_id, _data) if name == "damage" => {
                if let Some(health) = world.get_component_mut::<Health>(*entity_id) {
                    // Parse damage amount from event data (simplified)
                    let damage = 10.0; // In real implementation, deserialize from data
                    health.current = (health.current - damage).max(0.0);
                    self.damage_total += damage;
                    
                    println!("  💔 HealthObserver: Entity {:?} took {:.1} damage (health: {:.1}/{:.1})", 
                        entity_id, damage, health.current, health.max);
                    
                    if health.current <= 0.0 {
                        println!("  ☠️  HealthObserver: Entity {:?} has died!", entity_id);
                    }
                }
            }
            EntityEvent::Custom(name, entity_id, _data) if name == "heal" => {
                if let Some(health) = world.get_component_mut::<Health>(*entity_id) {
                    let heal = 15.0; // In real implementation, deserialize from data
                    let old_health = health.current;
                    health.current = (health.current + heal).min(health.max);
                    self.heal_total += heal;
                    
                    println!("  💚 HealthObserver: Entity {:?} healed for {:.1} ({} -> {:.1}/{:.1})", 
                        entity_id, heal, old_health, health.current, health.max);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "HealthObserver"
    }
}

fn main() {
    println!("=== Event System and Observers Example ===\n");
    
    // Create world and observer registry
    let mut world = World::new();
    let mut observers = ObserverRegistry::new();
    
    // Register observers
    println!("Registering observers...");
    
    let lifecycle_observer = Box::new(LifecycleObserver::new());
    observers.register(lifecycle_observer, &mut world).unwrap();
    
    let health_observer = Box::new(HealthObserver::new());
    observers.register(health_observer, &mut world).unwrap();
    
    println!("Registered {} observers\n", observers.observer_count());
    
    // Spawn some entities with events
    println!("=== Spawning Entities with Events ===");
    
    for i in 0..5 {
        let entity = world.spawn_with_event((
            Position { x: i as f32, y: 0.0, z: 0.0 },
            Health { current: 100.0, max: 100.0 },
        ));
        
        println!("Spawned entity {:?} with Position and Health", entity);
    }
    
    // Process all queued events
    println!("\n=== Processing Spawn Events ===");
    world.process_events().unwrap();
    
    // Broadcast events to observers
    println!("\n=== Broadcasting to Observers ===");
    
    // Create some custom events to demonstrate the observer system
    // Note: In a real implementation, you'd use actual entity IDs from spawned entities
    let damage_event = EntityEvent::Custom("damage".to_string(), 
        archetype_ecs::entity::EntityId::null(), vec![]);
    observers.broadcast(&damage_event, &mut world).unwrap();
    
    let heal_event = EntityEvent::Custom("heal".to_string(), 
        archetype_ecs::entity::EntityId::null(), vec![]);
    observers.broadcast(&heal_event, &mut world).unwrap();
    
    // Despawn an entity (simplified)
    println!("\n=== Despawning Entity ===");
    println!("Would despawn an entity with Health and Position components");
    
    // Process despawn events
    println!("\n=== Processing Despawn Events ===");
    world.process_events().unwrap();
    
    // Show final state
    println!("\n=== Final State ===");
    println!("Total entities: {}", world.entity_count());
    
    println!("\n=== Key Concepts Demonstrated ===");
    println!("1. Entity Events: Automatic events for spawn/despawn/component changes");
    println!("2. Custom Events: User-defined events with custom data");
    println!("3. Observer Pattern: Decoupled event handling");
    println!("4. Lifecycle Callbacks: Observer setup and registration events");
    println!("5. Event-Driven Architecture: Loose coupling via events");
    
    println!("\n=== Example Complete ===");
}
