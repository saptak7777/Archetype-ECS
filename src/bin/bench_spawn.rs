#![allow(dead_code)]

use aaa_ecs::World;
use std::time::Instant;

#[derive(Debug, Clone)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone)]
struct Health(u32);

fn main() {
    println!("Running spawn benchmarks...");

    // Warm up
    let mut world = World::new();
    let start = Instant::now();
    for _ in 0..1000 {
        world
            .spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)))
            .unwrap();
    }
    println!("Warmup (1k entities): {:?}", start.elapsed());

    // Benchmark spawning with 2 components
    let mut world = World::new();
    let start = Instant::now();
    for _ in 0..10_000 {
        world
            .spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)))
            .unwrap();
    }
    println!("Spawn 10k entities (2 components): {:?}", start.elapsed());

    // Benchmark spawning with 3 components
    let mut world = World::new();
    let start = Instant::now();
    for _ in 0..10_000 {
        world
            .spawn((
                Position(1.0, 2.0, 3.0),
                Velocity(1.0, 0.0, 0.0),
                Health(100),
            ))
            .unwrap();
    }
    println!("Spawn 10k entities (3 components): {:?}", start.elapsed());

    // Benchmark mixed spawning
    let mut world = World::new();
    let start = Instant::now();
    for i in 0..10_000 {
        if i % 2 == 0 {
            world
                .spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)))
                .unwrap();
        } else {
            world
                .spawn((
                    Position(1.0, 2.0, 3.0),
                    Velocity(1.0, 0.0, 0.0),
                    Health(100),
                ))
                .unwrap();
        }
    }
    println!("Spawn 10k entities (mixed): {:?}", start.elapsed());
}
