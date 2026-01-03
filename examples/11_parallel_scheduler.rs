//! Example: Parallel Systems with Phase 4 Scheduler

use archetype_ecs::*;

// Components
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for parallel scheduler
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for parallel scheduler
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Example component for parallel scheduler
struct Health {
    current: i32,
    max: i32,
}

// Example System
struct MovementSystem;

impl System for MovementSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(ComponentId::of::<Position>());
        access.writes.push(ComponentId::of::<Velocity>());
        access
    }

    fn name(&self) -> &'static str {
        "movement_system"
    }

    fn run(
        &mut self,
        _world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> Result<()> {
        println!("Running movement system");
        // In real implementation: query and update entities
        Ok(())
    }
}

struct HealthSystem;

impl System for HealthSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(ComponentId::of::<Health>());
        access
    }

    fn name(&self) -> &'static str {
        "health_system"
    }

    fn run(
        &mut self,
        _world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> Result<()> {
        println!("Running health system");
        Ok(())
    }
}

struct RenderSystem;

impl System for RenderSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(ComponentId::of::<Position>());
        access.reads.push(ComponentId::of::<Velocity>());
        access
    }

    fn name(&self) -> &'static str {
        "render_system"
    }

    fn run(
        &mut self,
        _world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> Result<()> {
        println!("Running render system");
        Ok(())
    }
}

fn main() -> Result<()> {
    println!("=== AAA ECS Phase 4: Parallel Scheduler Demo ===\n");

    // Create world
    let mut world = World::new();

    // Spawn entities
    let entity1 = world.spawn_entity((
        Position { x: 0.0, y: 0.0 },
        Velocity { x: 1.0, y: 0.5 },
        Health {
            current: 100,
            max: 100,
        },
    ));

    let entity2 = world.spawn_entity((
        Position { x: 10.0, y: 5.0 },
        Velocity { x: -0.5, y: 1.0 },
        Health {
            current: 75,
            max: 100,
        },
    ));

    println!("Spawned entities: {:?}, {:?}\n", entity1, entity2);

    // Create schedule and add systems
    let mut schedule = Schedule::new()
        .with_system(Box::new(MovementSystem))
        .with_system(Box::new(HealthSystem))
        .with_system(Box::new(RenderSystem))
        .build()?;

    println!("Schedule built successfully!");
    println!();

    // Create executor
    let mut executor = Executor::new(&mut schedule);

    // Execute frames
    println!("Executing 3 frames:\n");
    for frame in 0..3 {
        println!("Frame {}", frame);
        executor.execute_frame_parallel(&mut world)?;
        println!();
    }

    println!("=== Demo Complete ===");

    Ok(())
}
