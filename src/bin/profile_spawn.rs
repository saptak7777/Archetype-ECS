#![allow(dead_code, unused_imports)]

use archetype_ecs::World;
use std::{fs::File, time::Instant};

#[cfg(feature = "profiling")]
use tracing_subscriber::{self, prelude::*};

#[derive(Debug, Clone)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone)]
struct Health(u32);

#[cfg(feature = "profiling")]
#[tracing::instrument(skip(world))]
fn profile_spawns(world: &mut World, count: usize) {
    let _span = tracing::info_span!("spawn_loop", count = count).entered();
    for i in 0..count {
        if i % 1_000 == 0 {
            tracing::info!("Spawning entity {}/{}", i, count);
        }
        world.spawn_entity((
            Position(1.0, 2.0, 3.0),
            Velocity(1.0, 0.0, 0.0),
            Health(100),
        ));
    }
}

#[cfg(feature = "profiling")]
fn main() {
    // Set up tracing subscriber to write to a file
    let file = File::create("trace.json").unwrap();
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .init();

    let mut world = World::new();

    println!("Warming up...");
    {
        let _span = tracing::info_span!("warmup").entered();
        for _ in 0..1000 {
            world.spawn_entity((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)));
        }
    }

    println!("Profiling spawn with 3 components...");
    let start = Instant::now();
    profile_spawns(&mut world, 10_000);
    println!("Spawn 10k entities complete in: {:?}", start.elapsed());
}

#[cfg(not(feature = "profiling"))]
fn main() {
    println!("profile_spawn binary requires --features profiling");
}
