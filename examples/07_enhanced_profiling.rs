//! Enhanced Profiling Example
//!
//! This example demonstrates the new profiling features:
//! - Detailed profiling stats with per-entity timing
//! - CSV export for analysis
//! - Enhanced profiling summary with performance metrics

use archetype_ecs::{System, SystemAccess, World};
use std::time::Duration;

#[derive(Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Health {
    current: f32,
    max: f32,
}

// System that simulates physics calculations
struct PhysicsSystem;

impl System for PhysicsSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access
            .reads
            .push(archetype_ecs::system::ComponentId::of::<Velocity>());
        access
            .writes
            .push(archetype_ecs::system::ComponentId::of::<Position>());
        access
    }

    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }

    fn run(
        &mut self,
        world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> Result<(), archetype_ecs::error::EcsError> {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(1));

        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
            pos.x += vel.x;
            pos.y += vel.y;
        }

        Ok(())
    }
}

// System that simulates AI calculations
struct AISystem;

impl System for AISystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access
            .reads
            .push(archetype_ecs::system::ComponentId::of::<Position>());
        access
            .reads
            .push(archetype_ecs::system::ComponentId::of::<Health>());
        access
            .writes
            .push(archetype_ecs::system::ComponentId::of::<Health>());
        access
    }

    fn name(&self) -> &'static str {
        "AISystem"
    }

    fn run(
        &mut self,
        world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> Result<(), archetype_ecs::error::EcsError> {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(2));

        for health in world.query_mut::<&mut Health>().iter() {
            if health.current < health.max {
                health.current = (health.current + 1.0).min(health.max);
            }
        }

        Ok(())
    }
}

// System that simulates rendering
struct RenderSystem;

impl System for RenderSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access
            .reads
            .push(archetype_ecs::system::ComponentId::of::<Position>());
        access
            .reads
            .push(archetype_ecs::system::ComponentId::of::<Velocity>());
        access
            .reads
            .push(archetype_ecs::system::ComponentId::of::<Health>());
        access
    }

    fn name(&self) -> &'static str {
        "RenderSystem"
    }

    fn run(
        &mut self,
        _world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> Result<(), archetype_ecs::error::EcsError> {
        // Simulate some work
        std::thread::sleep(Duration::from_millis(3));

        // In a real render system, you'd actually render entities
        println!("  Rendering frame...");

        Ok(())
    }
}

fn main() {
    println!("=== Enhanced Profiling Example ===\n");

    // Create world and spawn entities
    let mut world = World::new();

    println!("Spawning entities...");
    for i in 0..1000 {
        world.spawn_entity((
            Position {
                x: i as f32,
                y: i as f32,
            },
            Velocity { x: 0.1, y: 0.05 },
            Health {
                current: 50.0,
                max: 100.0,
            },
        ));
    }

    println!("Spawned {} entities\n", world.entity_count());

    // Create schedule with systems
    let mut schedule = archetype_ecs::Schedule::new()
        .with_system(Box::new(PhysicsSystem))
        .with_system(Box::new(AISystem))
        .with_system(Box::new(RenderSystem))
        .build()
        .expect("build schedule");

    // Create executor
    let mut executor = archetype_ecs::Executor::new(&mut schedule);

    println!("Running frame with profiling...");

    // Execute frame
    if let Err(e) = executor.execute_frame(&mut world) {
        println!("Error: {e:?}");
    }

    // Demonstrate enhanced profiling features
    println!("\n=== Enhanced Profiling Features ===");

    // Get detailed profiling stats
    let stats = executor.profiling_stats(&world);
    println!("Detailed Stats:");
    println!("  Frame Time: {:?}", stats.total_frame_time);
    println!("  Systems: {}", stats.system_timings.len());
    println!("  Entities: {}", stats.entity_count);
    println!("  Time/Entity: {:.6} ms", stats.time_per_entity * 1000.0);
    println!("  Memory: {} bytes", stats.memory_usage);
    println!("  Systems/sec: {:.1}", stats.systems_per_second);

    // Print enhanced summary
    println!();
    executor.print_profiling_summary(&world);

    // Export profiling data to CSV
    println!("\nExporting profiling data to CSV...");
    if let Err(e) = executor.export_profiling_csv(&world, "profiling_data.csv") {
        println!("Failed to export CSV: {e:?}");
    } else {
        println!("Successfully exported to profiling_data.csv");
    }

    println!("\n=== Example Complete ===");
    println!("Open profiling_data.csv in a spreadsheet application to analyze performance!");
}
