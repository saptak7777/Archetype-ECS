use archetype_ecs::{DataResource, ResourceManager, TextureResource};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn create_test_resources(count: usize) -> Vec<DataResource> {
    (0..count)
        .map(|i| DataResource::new(format!("resource_{i}.bin"), vec![0u8; 1024]))
        .collect()
}

fn bench_load_100_resources(c: &mut Criterion) {
    c.bench_function("load_100_resources", |b| {
        b.iter(|| {
            let mut manager = ResourceManager::new(1024 * 1024 * 10);
            let resources = create_test_resources(100);

            for (i, resource) in resources.into_iter().enumerate() {
                let path = format!("resource_{i}.bin");
                manager.load(&path, resource).unwrap();
            }

            black_box(manager);
        })
    });
}

fn bench_load_and_get_resources(c: &mut Criterion) {
    c.bench_function("load_and_get_resources", |b| {
        b.iter(|| {
            let mut manager = ResourceManager::new(1024 * 1024 * 10);
            let resources = create_test_resources(50);

            for (i, resource) in resources.into_iter().enumerate() {
                let path = format!("resource_{i}.bin");
                manager.load(&path, resource).unwrap();
            }

            // Access resources
            for i in 0..50 {
                let path = format!("resource_{i}.bin");
                black_box(manager.get(&path));
            }
        })
    });
}

fn bench_texture_load(c: &mut Criterion) {
    c.bench_function("texture_load_256x256", |b| {
        b.iter(|| {
            let mut manager = ResourceManager::new(1024 * 1024 * 10);

            for i in 0..10 {
                let path = format!("texture_{i}.png");
                let texture =
                    TextureResource::new(path.clone(), 256, 256, vec![0u8; 256 * 256 * 4]);
                manager.load(&path, texture).unwrap();
            }

            black_box(manager);
        })
    });
}

fn bench_memory_utilization(c: &mut Criterion) {
    c.bench_function("memory_utilization_check", |b| {
        let mut manager = ResourceManager::new(1024 * 1024);

        for i in 0..100 {
            let path = format!("resource_{i}.bin");
            let resource = DataResource::new(path.clone(), vec![0u8; 1024]);
            manager.load(&path, resource).unwrap();
        }

        b.iter(|| {
            black_box(manager.get_memory_utilization());
        });
    });
}

criterion_group!(
    benches,
    bench_load_100_resources,
    bench_load_and_get_resources,
    bench_texture_load,
    bench_memory_utilization
);
criterion_main!(benches);
