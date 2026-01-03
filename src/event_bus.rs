use crate::error::Result;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Trait for any event type in the global event bus
pub trait Event: Send + Sync + 'static {
    /// Get the TypeId of this event
    fn event_type_id(&self) -> TypeId;

    /// Downcast to concrete type
    fn as_any(&self) -> &dyn Any;

    /// Event name for debugging
    fn event_name(&self) -> &str {
        "UnnamedEvent"
    }

    /// Validate event data (e.g. non-negative damage)
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Subscriber that listens for events
pub trait EventSubscriber: Send + Sync {
    /// Called when an event is published
    fn on_event(&mut self, event: &dyn Event) -> Result<()>;

    /// Get subscriber name for debugging
    fn name(&self) -> &str {
        "UnnamedSubscriber"
    }

    /// Check if this subscriber can handle the event
    fn can_handle(&self, _event_type: TypeId) -> bool {
        true // Default: handle all events
    }
}

/// Trait representing a type-erased event queue
trait EventStorage: Any + Send + Sync {
    #[allow(dead_code)]
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn process(&mut self, subscribers: &mut [Box<dyn EventSubscriber>]) -> Result<()>;
    fn clear(&mut self);
    fn len(&self) -> usize;
}

/// Contiguous storage for events of a specific type
struct TypedEventQueue<T: Event> {
    events: Vec<T>,
}

impl<T: Event> EventStorage for TypedEventQueue<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn process(&mut self, subscribers: &mut [Box<dyn EventSubscriber>]) -> Result<()> {
        for event in &self.events {
            for subscriber in subscribers.iter_mut() {
                subscriber.on_event(event)?;
            }
        }
        Ok(())
    }
    fn clear(&mut self) {
        self.events.clear();
    }
    fn len(&self) -> usize {
        self.events.len()
    }
}

/// Central event bus for pub/sub communication
pub struct EventBus {
    queues: HashMap<TypeId, Box<dyn EventStorage>>,
    subscribers: HashMap<TypeId, Vec<Box<dyn EventSubscriber>>>,
    processed_count: u64,
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        Self {
            queues: HashMap::new(),
            subscribers: HashMap::new(),
            processed_count: 0,
        }
    }

    /// Subscribe to specific event type
    pub fn subscribe<E: Event + 'static>(&mut self, subscriber: Box<dyn EventSubscriber>) {
        let type_id = TypeId::of::<E>();
        self.subscribers
            .entry(type_id)
            .or_default()
            .push(subscriber);
    }

    /// Subscribe to all event types (catches everything)
    /// Note: This is now slightly more expensive as it requires bridging
    pub fn subscribe_all(&mut self, subscriber: Box<dyn EventSubscriber>) {
        let type_id = TypeId::of::<()>(); // Wildcard
        self.subscribers
            .entry(type_id)
            .or_default()
            .push(subscriber);
    }

    /// Publish concrete event (Zero-Allocation!)
    pub fn publish_event<E: Event + 'static>(&mut self, event: E) -> Result<()> {
        let type_id = TypeId::of::<E>();
        let queue = self
            .queues
            .entry(type_id)
            .or_insert_with(|| Box::new(TypedEventQueue::<E> { events: Vec::new() }));

        let typed_queue = queue
            .as_any_mut()
            .downcast_mut::<TypedEventQueue<E>>()
            .unwrap();
        typed_queue.events.push(event);
        Ok(())
    }

    /// Process all queued events
    pub fn process_events(&mut self) -> Result<()> {
        let wildcard_type = TypeId::of::<()>();

        for (type_id, queue) in self.queues.iter_mut() {
            // 1. Process specific subscribers
            if let Some(subs) = self.subscribers.get_mut(type_id) {
                queue.process(subs)?;
            }

            // 2. Process wildcard subscribers
            if let Some(wildcard_subs) = self.subscribers.get_mut(&wildcard_type) {
                queue.process(wildcard_subs)?;
            }

            self.processed_count += queue.len() as u64;
            queue.clear();
        }

        Ok(())
    }

    /// Get number of queued events
    pub fn queue_size(&self) -> usize {
        self.queues.values().map(|q| q.len()).sum()
    }

    /// Get total processed events
    pub fn processed_count(&self) -> u64 {
        self.processed_count
    }

    /// Clear all queued events
    pub fn clear_queue(&mut self) {
        for queue in self.queues.values_mut() {
            queue.clear();
        }
    }

    /// Get subscriber count for event type
    pub fn subscriber_count(&self, event_type: TypeId) -> usize {
        self.subscribers
            .get(&event_type)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }

    /// Get total subscriber count
    pub fn total_subscribers(&self) -> usize {
        self.subscribers.values().map(|subs| subs.len()).sum()
    }

    /// Remove all subscribers
    pub fn clear_subscribers(&mut self) {
        self.subscribers.clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct TestEvent;
    impl Event for TestEvent {
        fn event_type_id(&self) -> TypeId {
            TypeId::of::<Self>()
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct TestSubscriber {
        call_count: Arc<Mutex<usize>>,
    }

    impl EventSubscriber for TestSubscriber {
        fn on_event(&mut self, _event: &dyn Event) -> Result<()> {
            *self.call_count.lock().unwrap() += 1;
            Ok(())
        }
    }

    #[test]
    fn test_publish_and_process() {
        let mut bus = EventBus::new();
        let count = Arc::new(Mutex::new(0));

        let subscriber = TestSubscriber {
            call_count: count.clone(),
        };

        bus.subscribe::<TestEvent>(Box::new(subscriber));
        bus.publish_event(TestEvent).unwrap();

        assert_eq!(bus.queue_size(), 1);
        bus.process_events().unwrap();

        assert_eq!(*count.lock().unwrap(), 1);
    }

    #[test]
    fn test_multiple_subscribers() {
        let mut bus = EventBus::new();
        let count1 = Arc::new(Mutex::new(0));
        let count2 = Arc::new(Mutex::new(0));

        bus.subscribe::<TestEvent>(Box::new(TestSubscriber {
            call_count: count1.clone(),
        }));

        bus.subscribe::<TestEvent>(Box::new(TestSubscriber {
            call_count: count2.clone(),
        }));

        bus.publish_event(TestEvent).unwrap();
        bus.process_events().unwrap();

        assert_eq!(*count1.lock().unwrap(), 1);
        assert_eq!(*count2.lock().unwrap(), 1);
    }
}
