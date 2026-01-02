use archetype_ecs::{Children, GlobalTransform, LocalTransform, Quat, Vec3, World};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bench_flat_entities(c: &mut Criterion) {
    c.bench_function("flat_1000_entities", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..1000 {
                black_box(world.spawn((LocalTransform::identity(), GlobalTransform::identity())));
            }
        })
    });
}

fn bench_hierarchy_creation(c: &mut Criterion) {
    c.bench_function("hierarchy_1_root_100_children", |b| {
        b.iter(|| {
            let mut world = World::new();
            let _root = black_box(world.spawn((
                LocalTransform::identity(),
                GlobalTransform::identity(),
                Children::new(),
            )));

            for _ in 0..100 {
                black_box(world.spawn((LocalTransform::identity(), GlobalTransform::identity())));
            }
        })
    });
}

fn bench_hierarchy_deep_tree(c: &mut Criterion) {
    c.bench_function("hierarchy_deep_20_levels", |b| {
        b.iter(|| {
            let mut world = World::new();
            let mut _parent =
                black_box(world.spawn((LocalTransform::identity(), GlobalTransform::identity())));

            for _ in 0..20 {
                let child = black_box(
                    world.spawn((LocalTransform::identity(), GlobalTransform::identity())),
                );
                _parent = child;
            }
        })
    });
}

fn bench_transform_operations(c: &mut Criterion) {
    c.bench_function("transform_local_to_global", |b| {
        let parent = GlobalTransform {
            position: Vec3::new(100.0, 200.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let child = LocalTransform {
            position: Vec3::new(10.0, 20.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        b.iter(|| {
            black_box(GlobalTransform::from_local(&parent, &child));
        })
    });
}

fn bench_vec3_operations(c: &mut Criterion) {
    c.bench_function("vec3_add_multiply", |b| {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        b.iter(|| {
            let result = black_box(v1 + v2);
            black_box(result * 2.0);
        })
    });
}

criterion_group!(
    benches,
    bench_flat_entities,
    bench_hierarchy_creation,
    bench_hierarchy_deep_tree,
    bench_transform_operations,
    bench_vec3_operations
);
criterion_main!(benches);
