#![allow(dead_code)]

use archetype_ecs::World;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Debug, Clone)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone)]
struct Health(u32);

fn spawn_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_bench");

    // Benchmark spawning entities with 2 components
    group.bench_function("spawn_2_components", |b| {
        let mut world = World::new();
        b.iter(|| {
            for _ in 0..1000 {
                black_box(
                    world
                        .spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)))
                        .unwrap(),
                );
            }
        });
    });

    // Benchmark spawning entities with 3 components
    group.bench_function("spawn_3_components", |b| {
        let mut world = World::new();
        b.iter(|| {
            for _ in 0..1000 {
                black_box(
                    world
                        .spawn((
                            Position(1.0, 2.0, 3.0),
                            Velocity(1.0, 0.0, 0.0),
                            Health(100),
                        ))
                        .unwrap(),
                );
            }
        });
    });

    // Benchmark spawning mixed entities
    group.bench_function("spawn_mixed", |b| {
        let mut world = World::new();
        b.iter(|| {
            for i in 0..1000 {
                if i % 2 == 0 {
                    black_box(
                        world
                            .spawn((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)))
                            .unwrap(),
                    );
                } else {
                    black_box(
                        world
                            .spawn((
                                Position(1.0, 2.0, 3.0),
                                Velocity(1.0, 0.0, 0.0),
                                Health(100),
                            ))
                            .unwrap(),
                    );
                }
            }
        });
    });

    group.finish();
}

criterion_group!(benches, spawn_benchmark);
criterion_main!(benches);
