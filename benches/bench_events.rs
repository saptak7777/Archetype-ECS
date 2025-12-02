use aaa_ecs::{Event, EventBus, EventSubscriber, Result};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::any::{Any, TypeId};

#[derive(Clone, Debug)]
struct TestEvent(u32);

impl Event for TestEvent {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct NoOpSubscriber;
impl EventSubscriber for NoOpSubscriber {
    fn on_event(&mut self, _event: &dyn Event) -> Result<()> {
        Ok(())
    }
}

fn bench_publish_1000_events(c: &mut Criterion) {
    c.bench_function("publish_1000_events", |b| {
        b.iter(|| {
            let mut bus = EventBus::new();
            for i in 0..1000 {
                bus.publish_event(TestEvent(i)).unwrap();
                black_box(());
            }
        })
    });
}

fn bench_process_1000_events_no_subscribers(c: &mut Criterion) {
    c.bench_function("process_1000_events_no_subs", |b| {
        b.iter(|| {
            let mut bus = EventBus::new();
            for i in 0..1000 {
                bus.publish_event(TestEvent(i)).unwrap();
            }
            bus.process_events().unwrap();
        })
    });
}

fn bench_process_1000_events_10_subscribers(c: &mut Criterion) {
    c.bench_function("process_1000_events_10_subs", |b| {
        b.iter(|| {
            let mut bus = EventBus::new();

            for _ in 0..10 {
                bus.subscribe::<TestEvent>(Box::new(NoOpSubscriber));
            }

            for i in 0..1000 {
                bus.publish_event(TestEvent(i)).unwrap();
            }
            bus.process_events().unwrap();
        })
    });
}

fn bench_process_1000_events_100_subscribers(c: &mut Criterion) {
    c.bench_function("process_1000_events_100_subs", |b| {
        b.iter(|| {
            let mut bus = EventBus::new();

            for _ in 0..100 {
                bus.subscribe::<TestEvent>(Box::new(NoOpSubscriber));
            }

            for i in 0..1000 {
                bus.publish_event(TestEvent(i)).unwrap();
            }
            bus.process_events().unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_publish_1000_events,
    bench_process_1000_events_no_subscribers,
    bench_process_1000_events_10_subscribers,
    bench_process_1000_events_100_subscribers
);
criterion_main!(benches);
