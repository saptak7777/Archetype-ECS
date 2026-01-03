#![cfg(feature = "profiling")]

use archetype_ecs::World;
use std::fs::File;
use tracing::info_span;

#[derive(Debug, Clone)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone)]
struct Health(u32);

// Function isolated for clearer profiling flamegraphs
#[tracing::instrument(skip_all, name = "spawn_workload")]
fn run_spawn_workload(world: &mut World, count: usize) {
    let batch_size = 1000;

    for i in (0..count).step_by(batch_size) {
        let _span = info_span!("spawn_batch", start_index = i, batch_size = batch_size).entered();

        for _ in 0..batch_size {
            world.spawn_entity((
                Position(1.0, 2.0, 3.0),
                Velocity(1.0, 0.0, 0.0),
                Health(100),
            ));
        }
    }
}

fn main() {
    // Set up tracing subscriber to write to a file
    let file = File::create("trace.json").unwrap();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_thread_ids(true)
        .with_thread_names(true)
        .json() // JSON output for easy import into profilers like Perfetto (via converter) or chrome://tracing
        .init();

    // Run spawn benchmarks with tracing
    let mut world = World::new();

    // Warm up
    {
        let _warmup = info_span!("warmup").entered();
        for _ in 0..1000 {
            world.spawn_entity((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)));
        }
    }

    // Profile workload
    run_spawn_workload(&mut world, 10_000);

    println!("Profiling complete. Output written to trace.json");
}
