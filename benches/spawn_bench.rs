use archetype_ecs::World;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Position(f32, f32, f32);

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Velocity(f32, f32, f32);

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Health(u32);

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Name(String);

fn spawn_simple_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_simple");

    group.bench_function("spawn_entity_2_components", |b| {
        b.iter_batched(
            || World::new(),
            |mut world| {
                for _ in 0..10_000 {
                    world.spawn_entity((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)));
                }
                world
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

fn spawn_heavy_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_heavy");

    group.bench_function("spawn_entity_4_components", |b| {
        b.iter_batched(
            || World::new(),
            |mut world| {
                for _ in 0..10_000 {
                    world.spawn_entity((
                        Position(1.0, 2.0, 3.0),
                        Velocity(1.0, 0.0, 0.0),
                        Health(100),
                        Name("Entity".to_string()),
                    ));
                }
                world
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

fn spawn_mixed_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_mixed");

    group.bench_function("spawn_entity_mixed", |b| {
        b.iter_batched(
            || World::new(),
            |mut world| {
                for i in 0..10_000 {
                    if i % 2 == 0 {
                        world.spawn_entity((Position(1.0, 2.0, 3.0), Velocity(1.0, 0.0, 0.0)));
                    } else {
                        world.spawn_entity((
                            Position(1.0, 2.0, 3.0),
                            Velocity(1.0, 0.0, 0.0),
                            Health(100),
                        ));
                    }
                }
                world
            },
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    spawn_simple_benchmark,
    spawn_heavy_benchmark,
    spawn_mixed_benchmark
);
criterion_main!(benches);
