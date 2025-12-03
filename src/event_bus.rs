use crate::error::Result;
use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};

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

/// Central event bus for pub/sub communication
pub struct EventBus {
    subscribers: HashMap<TypeId, Vec<Box<dyn EventSubscriber>>>,
    event_queue: VecDeque<Box<dyn Event>>,
    max_queue_size: usize,
    processed_events: u64,
}

impl EventBus {
    /// Create new event bus
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            event_queue: VecDeque::new(),
            max_queue_size: 10000,
            processed_events: 0,
        }
    }

    /// Create with custom max queue size
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            subscribers: HashMap::new(),
            event_queue: VecDeque::with_capacity(max_size),
            max_queue_size: max_size,
            processed_events: 0,
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
    pub fn subscribe_all(&mut self, subscriber: Box<dyn EventSubscriber>) {
        let type_id = TypeId::of::<()>(); // Use unit type as wildcard
        self.subscribers
            .entry(type_id)
            .or_default()
            .push(subscriber);
    }

    /// Publish event to queue
    pub fn publish(&mut self, event: Box<dyn Event>) -> Result<()> {
        if self.event_queue.len() < self.max_queue_size {
            self.event_queue.push_back(event);
            Ok(())
        } else {
            Err(crate::error::EcsError::EventQueueOverflow)
        }
    }

    /// Publish concrete event (convenience)
    pub fn publish_event<E: Event + 'static>(&mut self, event: E) -> Result<()> {
        self.publish(Box::new(event))
    }

    /// Process all queued events
    pub fn process_events(&mut self) -> Result<()> {
        while let Some(event) = self.event_queue.pop_front() {
            let event_type = event.event_type_id();

            // Get subscribers for this event type
            if let Some(subs) = self.subscribers.get_mut(&event_type) {
                for subscriber in subs.iter_mut() {
                    subscriber.on_event(event.as_ref())?;
                }
            }

            // Also notify wildcard subscribers
            let wildcard_type = TypeId::of::<()>();
            if let Some(wildcard_subs) = self.subscribers.get_mut(&wildcard_type) {
                for subscriber in wildcard_subs.iter_mut() {
                    subscriber.on_event(event.as_ref())?;
                }
            }

            self.processed_events += 1;
        }

        Ok(())
    }

    /// Get number of queued events
    pub fn queue_size(&self) -> usize {
        self.event_queue.len()
    }

    /// Get total processed events
    pub fn processed_count(&self) -> u64 {
        self.processed_events
    }

    /// Clear all queued events
    pub fn clear_queue(&mut self) {
        self.event_queue.clear();
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
    use parking_lot::Mutex;
    use std::sync::Arc;

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
            *self.call_count.lock() += 1;
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

        assert_eq!(*count.lock(), 1);
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

        assert_eq!(*count1.lock(), 1);
        assert_eq!(*count2.lock(), 1);
    }

    #[test]
    fn test_queue_overflow() {
        let mut bus = EventBus::with_capacity(2);

        bus.publish_event(TestEvent).unwrap();
        bus.publish_event(TestEvent).unwrap();

        // Third publish should fail
        assert!(bus.publish_event(TestEvent).is_err());
    }
}
