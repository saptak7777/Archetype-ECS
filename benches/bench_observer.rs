use archetype_ecs::{EntityEvent, Observer, StatisticsObserver, World};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[derive(Clone, Copy, Debug)]
struct Position {
    #[allow(dead_code)]
    x: f32,
    #[allow(dead_code)]
    y: f32,
}

struct NoOpObserver;
impl Observer for NoOpObserver {
    fn on_event(&mut self, _event: &EntityEvent, _world: &mut World) -> archetype_ecs::Result<()> {
        Ok(())
    }
    fn name(&self) -> &str {
        "NoOp"
    }
}

fn bench_spawn_no_observers(c: &mut Criterion) {
    c.bench_function("spawn_no_observers", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..100 {
                black_box(world.spawn((Position { x: 0.0, y: 0.0 },)).unwrap());
            }
        })
    });
}

fn bench_spawn_with_event_no_observers(c: &mut Criterion) {
    c.bench_function("spawn_with_event_no_observers", |b| {
        b.iter(|| {
            let mut world = World::new();
            for _ in 0..100 {
                black_box(
                    world
                        .spawn_with_event((Position { x: 0.0, y: 0.0 },))
                        .unwrap(),
                );
            }
            world.process_events().ok();
        })
    });
}

fn bench_spawn_with_noop_observer(c: &mut Criterion) {
    c.bench_function("spawn_with_noop_observer", |b| {
        b.iter(|| {
            let mut world = World::new();
            world.register_observer(Box::new(NoOpObserver)).ok();

            for _ in 0..100 {
                black_box(
                    world
                        .spawn_with_event((Position { x: 0.0, y: 0.0 },))
                        .unwrap(),
                );
            }
            world.process_events().ok();
        })
    });
}

fn bench_spawn_with_stats_observer(c: &mut Criterion) {
    c.bench_function("spawn_with_stats_observer", |b| {
        b.iter(|| {
            let mut world = World::new();
            world
                .register_observer(Box::new(StatisticsObserver::new()))
                .ok();

            for _ in 0..100 {
                black_box(
                    world
                        .spawn_with_event((Position { x: 0.0, y: 0.0 },))
                        .unwrap(),
                );
            }
            world.process_events().ok();
        })
    });
}

criterion_group!(
    benches,
    bench_spawn_no_observers,
    bench_spawn_with_event_no_observers,
    bench_spawn_with_noop_observer,
    bench_spawn_with_stats_observer
);
criterion_main!(benches);
