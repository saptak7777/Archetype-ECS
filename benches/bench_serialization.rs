use archetype_ecs::{EntityData, EntityIdData, WorldData};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap;

fn create_test_world(entity_count: usize) -> WorldData {
    let mut world = WorldData::new();

    for i in 0..entity_count {
        let mut components = HashMap::new();
        components.insert(
            "Position".to_string(),
            serde_json::json!({ "x": i as f32, "y": (i*2) as f32, "z": 0.0 }),
        );
        components.insert(
            "Health".to_string(),
            serde_json::json!({ "hp": 100.0, "max_hp": 100.0 }),
        );

        let entity = EntityData {
            id: EntityIdData {
                index: i as u32,
                generation: 0,
            },
            components,
        };

        world.add_entity(entity);
    }

    world
}

fn bench_serialize_100_entities_json(c: &mut Criterion) {
    c.bench_function("serialize_100_entities_json", |b| {
        b.iter(|| {
            let world = black_box(create_test_world(100));
            world.to_json_string()
        })
    });
}

fn bench_serialize_1000_entities_json(c: &mut Criterion) {
    c.bench_function("serialize_1000_entities_json", |b| {
        b.iter(|| {
            let world = black_box(create_test_world(1000));
            world.to_json_string()
        })
    });
}

fn bench_deserialize_1000_entities_json(c: &mut Criterion) {
    c.bench_function("deserialize_1000_entities_json", |b| {
        let world = create_test_world(1000);
        let json = world.to_json_string().unwrap();

        b.iter(|| WorldData::from_json_string(black_box(&json)))
    });
}

fn bench_serialize_100_entities_binary(c: &mut Criterion) {
    c.bench_function("serialize_100_entities_binary", |b| {
        b.iter(|| {
            let world = black_box(create_test_world(100));
            world.to_binary_bytes()
        })
    });
}

fn bench_deserialize_100_entities_binary(c: &mut Criterion) {
    c.bench_function("deserialize_100_entities_binary", |b| {
        let world = create_test_world(100);
        let bytes = world.to_binary_bytes().unwrap();

        b.iter(|| WorldData::from_binary_bytes(black_box(&bytes)))
    });
}

criterion_group!(
    benches,
    bench_serialize_100_entities_json,
    bench_serialize_1000_entities_json,
    bench_deserialize_1000_entities_json,
    bench_serialize_100_entities_binary,
    bench_deserialize_100_entities_binary
);
criterion_main!(benches);
