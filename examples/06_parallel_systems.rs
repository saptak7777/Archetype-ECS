//! Example: Parallel Systems with Phase 4 Scheduler

use archetype_ecs::*;

// Components
#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Health {
    current: i32,
    max: i32,
}

// Systems
#[derive(Debug)]
struct MovementSystem;

impl System for MovementSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(ComponentId::of::<Velocity>());
        access.writes.push(ComponentId::of::<Position>());
        access
    }

    fn name(&self) -> &'static str {
        "MovementSystem"
    }

    fn run(
        &mut self,
        world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> std::result::Result<(), archetype_ecs::error::EcsError> {
        println!("  MovementSystem running...");

        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
            pos.x += vel.x;
            pos.y += vel.y;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct HealthSystem;

impl System for HealthSystem {
    fn accesses(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(ComponentId::of::<Health>());
        access.writes.push(ComponentId::of::<Health>());
        access
    }

    fn name(&self) -> &'static str {
        "HealthSystem"
    }

    fn run(
        &mut self,
        world: &mut World,
        _commands: &mut archetype_ecs::command::CommandBuffer,
    ) -> std::result::Result<(), archetype_ecs::error::EcsError> {
        println!("  HealthSystem running...");

        for health in world.query_mut::<&mut Health>().iter() {
            health.current = (health.current + 1).min(health.max);
        }

        Ok(())
    }
}

fn main() {
    println!("=== Parallel Systems Example ===");

    // Create world and spawn entities
    let mut world = World::new();

    println!("Spawning entities...");
    for i in 0..1000 {
        world.spawn_entity((
            Position {
                x: i as f32,
                y: i as f32,
            },
            Velocity { x: 0.1, y: 0.0 },
            Health {
                current: 50,
                max: 100,
            },
        ));
    }

    println!("Spawned {} entities", world.entity_count());

    // Create schedule with systems
    let systems: Vec<Box<dyn archetype_ecs::System>> =
        vec![Box::new(MovementSystem), Box::new(HealthSystem)];
    let mut schedule = Schedule::from_systems(systems).unwrap();
    let mut executor = Executor::new(&mut schedule);

    println!("Running parallel systems...");
    if let Err(e) = executor.execute_frame(&mut world) {
        println!("Error: {:?}", e);
    }

    println!("=== Example Complete ===");
}
