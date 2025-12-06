use archetype_ecs::assets::{Asset, AssetCache};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use std::sync::Arc;

// Simple mock asset
#[derive(Debug, Clone)]
struct MockAsset {
    data: Vec<u8>,
}

impl Asset for MockAsset {
    fn memory_size(&self) -> usize {
        self.data.len()
    }
}

fn bench_asset_insert_evict(c: &mut Criterion) {
    c.bench_function("asset_insert_with_eviction", |b| {
        // limit size to force eviction
        let cache = AssetCache::new(1000);
        let asset = MockAsset { data: vec![0; 100] };
        let mut id = 0;

        b.iter(|| {
            // Each insert should trigger eviction of one old item
            cache.insert(black_box(id), asset.clone());
            id += 1;
        });
    });
}

fn bench_asset_concurrent_read(c: &mut Criterion) {
    let cache = Arc::new(AssetCache::new(1024 * 1024));

    // Pre-populate
    for i in 0..1000 {
        cache.insert(i, MockAsset { data: vec![0; 10] });
    }

    c.bench_function("asset_concurrent_read", |b| {
        b.iter(|| {
            // Simulate read which updates atomic stats and LRU timestamp
            // Lock-free read path
            cache.get::<MockAsset>(black_box(500));
        });
    });
}

criterion_group!(
    benches,
    bench_asset_insert_evict,
    bench_asset_concurrent_read
);
criterion_main!(benches);
