use aaa_ecs::World;
use std::{fs::File, time::Instant};

#[derive(Debug, Clone)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone)]
struct Health(u32);

fn main() {
    // Set up tracing subscriber to write to a file
    let file = File::create("trace.json").unwrap();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Run spawn benchmarks with tracing
    let mut world = World::new();
    
    // Warm up
    for _ in 0..1000 {
        world.spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0))).unwrap();
    }

    // Profile spawn with 3 components
    let start = Instant::now();
    for i in 0..10_000 {
        world.spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0), Health(100))).unwrap();
    }
    println!("Spawn 10k entities: {:?}", start.elapsed());
}
