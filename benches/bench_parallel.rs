use archetype_ecs::{Executor, Schedule, System, SystemAccess, World};
use criterion::{criterion_group, criterion_main, Criterion};
use std::any::TypeId;

struct HeavySystem {
    id: usize,
    reads: Vec<TypeId>,
    writes: Vec<TypeId>,
}

impl System for HeavySystem {
    fn name(&self) -> &'static str {
        "HeavySystem"
    }

    fn access(&self) -> SystemAccess {
        SystemAccess {
            reads: self.reads.clone(),
            writes: self.writes.clone(),
        }
    }

    fn run(&mut self, _world: &mut World) -> archetype_ecs::Result<()> {
        // Simulate work
        let mut _x = 0;
        for i in 0..1_000_000 {
            _x += i;
            std::hint::black_box(());
        }
        Ok(())
    }
}

fn bench_parallel_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_execution");

    // Create 100 independent systems (should scale perfectly)
    let mut systems: Vec<Box<dyn System>> = Vec::new();
    for i in 0..100 {
        systems.push(Box::new(HeavySystem {
            id: i,
            reads: vec![], // No conflicts
            writes: vec![],
        }));
    }

    let schedule = Schedule::from_systems(systems).unwrap();
    let mut executor = Executor::new(schedule);
    let mut world = World::new();

    group.bench_function("sequential", |b| {
        b.iter(|| {
            executor.execute_frame(&mut world).unwrap();
        })
    });

    group.bench_function("parallel", |b| {
        b.iter(|| {
            executor.execute_frame_parallel(&mut world).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_parallel_execution);
criterion_main!(benches);
