//! Profiling Basics Example
//!
//! This example shows how to enable and use the built-in profiling
//! instrumentation in Archetype ECS.

use archetype_ecs::World;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct Pos(f32, f32);
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct Vel(f32, f32);

fn main() {
    println!("=== Archetype ECS Profiling Basics ===\n");
    println!("To see profiling output, run with:");
    println!("cargo run --example 16_profiling_basics --features profiling\n");

    let mut world = World::new();

    // The 'profiling' feature enables tracing spans for core operations
    println!("Spawning entities (instrumented)...");
    for i in 0..1000 {
        world.spawn_entity((Pos(i as f32, 0.0), Vel(1.0, 0.0)));
    }

    println!("Running queries (instrumented)...");
    // Queries also have spans for archetype matching and iteration
    let mut query = world.query_mut::<(&mut Pos, &Vel)>();
    for (pos, vel) in query.iter() {
        pos.0 += vel.0;
    }

    println!("Simulation step complete.");

    // In a real application, you would set up a tracing subscriber
    // to collect and view these spans.

    #[cfg(feature = "profiling")]
    {
        println!("\n[PROFILING ENABLED]");
        println!("Core operations are being timed and categorized by 'tracing'.");
    }

    #[cfg(not(feature = "profiling"))]
    {
        println!("\n[PROFILING DISABLED]");
        println!("Note: Profiling spans are zero-cost when the feature is disabled.");
    }

    // Wait a bit to simulate a game loop
    std::thread::sleep(Duration::from_millis(100));
}
