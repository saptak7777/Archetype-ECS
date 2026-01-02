//! Example 3: System Dependencies and Stages
//! 
//! This example demonstrates:
//! - Creating named stages with dependencies
//! - Adding systems to specific stages
//! - Understanding system execution order
//! - Using the App API for system management

use archetype_ecs::{World, App, System, SystemAccess};

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

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for system dependencies
struct Transform {
    #[allow(dead_code)] // Transformation matrix for demo purposes
    matrix: [[f32; 4]; 4], // 4x4 transformation matrix
}

// System that updates positions based on velocity
struct MovementSystem;

impl System for MovementSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Velocity>());
        access.writes.push(std::any::TypeId::of::<Position>());
        access
    }

    fn name(&self) -> &'static str {
        "MovementSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  Running MovementSystem...");
        
        let mut moved_count = 0;
        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
            pos.x += vel.x;
            pos.y += vel.y;
            pos.z += vel.z;
            moved_count += 1;
        }
        
        println!("    Moved {moved_count} entities");
        Ok(())
    }
}

// System that calculates transforms from positions
struct TransformSystem;

impl System for TransformSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Position>());
        access.writes.push(std::any::TypeId::of::<Transform>());
        access
    }

    fn name(&self) -> &'static str {
        "TransformSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  Running TransformSystem...");
        
        let mut transformed_count = 0;
        for _pos in world.query::<&Position>().iter() {
            // In a real system, you'd calculate actual transforms
            // For this example, we'll just count
            transformed_count += 1;
        }
        
        println!("    Calculated transforms for {transformed_count} entities");
        Ok(())
    }
}

// System that renders entities
struct RenderSystem;

impl System for RenderSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Position>());
        access.reads.push(std::any::TypeId::of::<Transform>());
        access
    }

    fn name(&self) -> &'static str {
        "RenderSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  Running RenderSystem...");
        
        let mut rendered_count = 0;
        for (_pos, _transform) in world.query::<(&Position, &Transform)>().iter() {
            // In a real system, you'd actually render
            // For this example, we'll just count
            rendered_count += 1;
        }
        
        println!("    Rendered {rendered_count} entities");
        Ok(())
    }
}

// System that cleans up old entities
struct CleanupSystem;

impl System for CleanupSystem {
    fn access(&self) -> SystemAccess {
        SystemAccess::empty()
    }

    fn name(&self) -> &'static str {
        "CleanupSystem"
    }

    fn run(&mut self, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  Running CleanupSystem...");
        
        // In a real system, you'd remove old entities
        println!("    Cleaned up old entities");
        Ok(())
    }
}

fn main() {
    println!("=== System Dependencies and Stages Example ===\n");
    
    // Create app with stages
    let mut app = App::new();
    
    // Define stages with dependencies
    println!("Setting up stages with dependencies...");
    
    // Create stages
    app.schedule.add_stage("physics").unwrap();
    app.schedule.add_stage("transform").unwrap();
    app.schedule.add_stage("render").unwrap();
    app.schedule.add_stage("cleanup").unwrap();
    
    // Define dependencies: physics -> transform -> render -> cleanup
    app.schedule.add_stage_dependency("transform", "physics").unwrap();
    app.schedule.add_stage_dependency("render", "transform").unwrap();
    app.schedule.add_stage_dependency("cleanup", "render").unwrap();
    
    println!("Stage dependencies:");
    println!("  physics -> transform -> render -> cleanup\n");
    
    // Add systems to stages
    println!("Adding systems to stages...");
    
    // Add systems to appropriate stages
    app.schedule.add_system_to_stage("physics", Box::new(MovementSystem)).unwrap();
    app.schedule.add_system_to_stage("transform", Box::new(TransformSystem)).unwrap();
    app.schedule.add_system_to_stage("render", Box::new(RenderSystem)).unwrap();
    app.schedule.add_system_to_stage("cleanup", Box::new(CleanupSystem)).unwrap();
    
    println!("Systems added:");
    println!("  physics: MovementSystem");
    println!("  transform: TransformSystem");
    println!("  render: RenderSystem");
    println!("  cleanup: CleanupSystem\n");
    
    // Spawn some test entities
    println!("Spawning test entities...");
    for i in 0..100 {
        app.world.spawn((
            Position { x: i as f32, y: 0.0, z: 0.0 },
            Velocity { x: 0.1, y: 0.0, z: 0.0 },
            Transform {
                matrix: [[1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0, 0.0, 0.0, 1.0]],
            },
        ));
    }
    
    println!("Spawned {} entities\n", app.world.entity_count());
    
    // Run one frame to demonstrate system execution
    println!("=== Running Systems (One Frame) ===");
    
    if let Err(e) = app.update() {
        println!("Error running systems: {e:?}");
    }
    
    println!("\n=== Key Concepts Demonstrated ===");
    println!("1. Named Stages: Organize systems into logical groups");
    println!("2. Stage Dependencies: Control execution order between stages");
    println!("3. System Registration: Add systems to specific stages");
    println!("4. Execution Order: Systems run in dependency order");
    println!("5. App API: High-level interface for system management");
    
    println!("\n=== Example Complete ===");
}
