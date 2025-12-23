#![allow(dead_code)]
//! Benchmarks for Phase 1 performance
//!
//! Run with: cargo bench
//!
//! This benchmark suite measures core ECS operations:
//! - Entity spawning
//! - Entity despawning
//! - Entity lookup
//! - Archetype operations

use archetype_ecs::{archetype::Archetype, QueryState, World as AaaWorld};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hecs::World as HecsWorld;

#[derive(Debug, Copy, Clone)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Debug, Copy, Clone)]
struct Health(u32);

#[derive(Debug, Copy, Clone)]
struct Damage(f32);

// Bench: Spawning entities with different component counts
fn bench_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn");

    // Spawn single component
    group.bench_function("aaa_spawn_1k_single_component", |b| {
        b.iter(|| {
            let mut world = AaaWorld::new();
            for i in 0..1_000 {
                let _ = world.spawn((Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },));
            }
        });
    });
    group.bench_function("hecs_spawn_1k_single_component", |b| {
        b.iter(|| {
            let mut world = HecsWorld::new();
            for i in 0..1_000 {
                world.spawn((Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },));
            }
        });
    });

    // Spawn two components
    group.bench_function("aaa_spawn_1k_two_components", |b| {
        b.iter(|| {
            let mut world = AaaWorld::new();
            for i in 0..1_000 {
                let _ = world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
            }
        });
    });
    group.bench_function("hecs_spawn_1k_two_components", |b| {
        b.iter(|| {
            let mut world = HecsWorld::new();
            for i in 0..1_000 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
            }
        });
    });

    // Spawn three components
    group.bench_function("aaa_spawn_1k_three_components", |b| {
        b.iter(|| {
            let mut world = AaaWorld::new();
            for i in 0..1_000 {
                let _ = world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                ));
            }
        });
    });
    group.bench_function("hecs_spawn_1k_three_components", |b| {
        b.iter(|| {
            let mut world = HecsWorld::new();
            for i in 0..1_000 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                ));
            }
        });
    });

    // Spawn four components
    group.bench_function("aaa_spawn_1k_four_components", |b| {
        b.iter(|| {
            let mut world = AaaWorld::new();
            for i in 0..1_000 {
                let _ = world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                    Damage(10.5),
                ));
            }
        });
    });
    group.bench_function("hecs_spawn_1k_four_components", |b| {
        b.iter(|| {
            let mut world = HecsWorld::new();
            for i in 0..1_000 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                    Damage(10.5),
                ));
            }
        });
    });

    group.finish();
}

// Bench: Spawning large batches
fn bench_spawn_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_large");

    for count in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("aaa_spawn_with_3_components", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut world = AaaWorld::new();
                    for i in 0..count {
                        let _ = world.spawn((
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Velocity {
                                x: 1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        ));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hecs_spawn_with_3_components", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut world = HecsWorld::new();
                    for i in 0..count {
                        world.spawn((
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Velocity {
                                x: 1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        ));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("aaa_spawn_batch_with_3_components", count),
            count,
            |b, &count| {
                b.iter(|| {
                    let mut world = AaaWorld::new();
                    let bundles = (0..count).map(|i| {
                        (
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Velocity {
                                x: 1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        )
                    });
                    // Measure batch spawn
                    let _ = world.spawn_batch(bundles);
                });
            },
        );
    }

    group.finish();
}

// Bench: Entity lookup performance
fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");

    for count in [1_000, 10_000, 100_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("aaa_lookup_entities", count),
            count,
            |b, &count| {
                let mut world = AaaWorld::new();
                let entities: Vec<_> = (0..count)
                    .map(|i| {
                        world.spawn((
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        ))
                    })
                    .collect();

                b.iter(|| {
                    for &entity in &entities {
                        black_box(world.get_entity_location(entity));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hecs_lookup_entities", count),
            count,
            |b, &count| {
                let mut world = HecsWorld::new();
                let entities: Vec<_> = (0..count)
                    .map(|i| {
                        world.spawn((
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        ))
                    })
                    .collect();

                b.iter(|| {
                    for &entity in &entities {
                        black_box(world.get::<&Position>(entity).ok());
                    }
                });
            },
        );
    }

    group.finish();
}

// Bench: Despawn performance
fn bench_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("despawn");

    group.bench_function("aaa_despawn_1k_entities", |b| {
        b.iter_batched(
            || {
                let mut world = AaaWorld::new();
                let entities: Vec<_> = (0..1_000)
                    .map(|i| {
                        world.spawn((
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        ))
                    })
                    .collect();
                (world, entities)
            },
            |(mut world, entities)| {
                for entity in entities {
                    let _ = world.despawn(entity);
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("hecs_despawn_1k_entities", |b| {
        b.iter_batched(
            || {
                let mut world = HecsWorld::new();
                let entities: Vec<_> = (0..1_000)
                    .map(|i| {
                        world.spawn((
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                            Health(100),
                        ))
                    })
                    .collect();
                (world, entities)
            },
            |(mut world, entities)| {
                for entity in entities {
                    let _ = world.despawn(entity);
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

// Bench: Archetype operations
fn bench_archetype_segregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("archetype");

    group.bench_function("aaa_archetype_segregation_1k", |b| {
        b.iter(|| {
            let mut world = AaaWorld::new();

            for i in 0..250 {
                let _ = world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
            }

            for i in 0..250 {
                let _ = world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                ));
            }

            for i in 0..250 {
                let _ = world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                ));
            }

            for i in 0..250 {
                let _ = world.spawn((Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },));
            }
        });
    });

    group.bench_function("hecs_archetype_segregation_1k", |b| {
        b.iter(|| {
            let mut world = HecsWorld::new();

            for i in 0..250 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                ));
            }

            for i in 0..250 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                ));
            }

            for i in 0..250 {
                world.spawn((
                    Position {
                        x: i as f32,
                        y: 0.0,
                        z: 0.0,
                    },
                    Velocity {
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    },
                    Health(100),
                ));
            }

            for i in 0..250 {
                world.spawn((Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },));
            }
        });
    });

    group.finish();
}

// Bench: Query creation and steady-state iteration
fn bench_query_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("query");

    group.bench_function("aaa_query_state_creation_10k", |b| {
        let mut world = AaaWorld::new();
        for i in 0..10_000 {
            let _ = world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Health(100),
            ));
        }

        b.iter(|| {
            // Intentionally measure construction cost by recreating each iteration
            let _state = QueryState::<(&Position, &Velocity)>::new(&world);
        });
    });

    group.bench_function("aaa_query_iteration_cached_100k", |b| {
        let mut world = AaaWorld::new();
        for i in 0..100_000 {
            let _ = world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Health(100),
            ));
        }

        // Initialize cache
        let _ = world
            .query_mut::<(&mut Position, &Velocity)>()
            .iter()
            .count();

        b.iter(|| {
            for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>() {
                pos.x += vel.x;
            }
        });
    });

    group.bench_function("aaa_query_iteration_simd_100k", |b| {
        let mut world = AaaWorld::new();
        for i in 0..100_000 {
            let _ = world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Health(100),
            ));
        }

        b.iter(|| {
            world
                .query_mut::<(&mut Position, &Velocity)>()
                .par_for_each_chunk(|chunk| {
                    // SAFETY: We access disjoint components
                    let archetype = chunk.archetype as *mut Archetype;
                    let range = chunk.entity_range.clone();

                    unsafe {
                        if let (Some(all_pos), Some(all_vel)) = (
                            (*archetype).get_component_slice_mut::<Position>(),
                            (*archetype).get_component_slice::<Velocity>(),
                        ) {
                            let positions = &mut all_pos[range.clone()];
                            let velocities = &all_vel[range];

                            for (pos, vel) in positions.iter_mut().zip(velocities.iter()) {
                                pos.x += vel.x;
                            }
                        }
                    }
                });
        });
    });

    group.bench_function("hecs_query_state_creation_10k", |b| {
        let mut world = HecsWorld::new();
        for i in 0..10_000 {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Health(100),
            ));
        }

        b.iter(|| {
            world.query::<(&Position, &Velocity)>().iter().count();
        });
    });

    group.bench_function("hecs_query_iteration_10k", |b| {
        let mut world = HecsWorld::new();
        for i in 0..10_000 {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Health(100),
            ));
        }

        let mut query = world.query::<(&Position, &Velocity)>();

        b.iter(|| {
            let mut count = 0;
            for _ in query.iter() {
                count += 1;
            }
            black_box(count);
        });
    });

    group.finish();
}

// Bench: Entity count statistics
fn bench_entity_count(c: &mut Criterion) {
    c.bench_function("aaa_entity_count_10k", |b| {
        let mut world = AaaWorld::new();
        for i in 0..10_000 {
            let _ = world.spawn((Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },));
        }

        b.iter(|| {
            black_box(world.entity_count());
        });
    });

    c.bench_function("hecs_entity_count_10k", |b| {
        let mut world = HecsWorld::new();
        for i in 0..10_000 {
            world.spawn((Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },));
        }

        b.iter(|| {
            black_box(world.len());
        });
    });
}
// Bench: Archetype count (AAA only; hecs lacks public archetype introspection)
fn bench_archetype_count(c: &mut Criterion) {
    c.bench_function("aaa_archetype_count_mixed", |b| {
        let mut world = AaaWorld::new();

        for i in 0..100 {
            let _ = world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ));
        }

        for i in 0..100 {
            let _ = world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Health(100),
            ));
        }

        for i in 0..100 {
            let _ = world.spawn((Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },));
        }

        b.iter(|| {
            black_box(world.archetype_count());
        });
    });
}

// Group all benchmarks
criterion_group!(
    benches,
    bench_spawn,
    bench_spawn_large,
    bench_lookup,
    bench_despawn,
    bench_archetype_segregation,
    bench_query_creation,
    bench_entity_count,
    bench_archetype_count
);

criterion_main!(benches);
