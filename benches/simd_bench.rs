use archetype_ecs::{QueryState, World};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Benchmark test struct for SIMD operations
struct Position {
    x: f32,
    #[allow(dead_code)] // Unused field for benchmark testing
    y: f32,
    #[allow(dead_code)] // Unused field for benchmark testing
    z: f32,
}

fn bench_simd_chunks(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_chunks");

    for &entity_count in &[1000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("simd_chunks", entity_count),
            &entity_count,
            |b, &entity_count| {
                b.iter(|| {
                    let mut world = World::new();
                    for _ in 0..entity_count {
                        world.spawn_entity((Position {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                        },));
                    }

                    let mut query = QueryState::<&mut Position>::new(&world);
                    let chunks = query.iter_simd_chunks::<Position>(&mut world);

                    for chunk in chunks {
                        for pos in chunk.iter() {
                            black_box(pos.x);
                        }
                    }

                    black_box(world);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_simd_chunks);
criterion_main!(benches);
