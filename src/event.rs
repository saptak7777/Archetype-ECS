use crate::entity::EntityId;
use std::any::TypeId;

/// Entity lifecycle events that trigger observers
#[derive(Clone, Debug)]
pub enum EntityEvent {
    /// Entity was spawned (created)
    Spawned(EntityId),

    /// Entity was despawned (destroyed)
    Despawned(EntityId),

    /// Component was added to entity
    ComponentAdded(EntityId, TypeId),

    /// Component was removed from entity
    ComponentRemoved(EntityId, TypeId),

    /// Custom event (name, entity_id, data)
    Custom(String, EntityId, Vec<u8>),
}

impl EntityEvent {
    /// Get the entity involved in this event
    pub fn entity_id(&self) -> EntityId {
        match self {
            EntityEvent::Spawned(id) => *id,
            EntityEvent::Despawned(id) => *id,
            EntityEvent::ComponentAdded(id, _) => *id,
            EntityEvent::ComponentRemoved(id, _) => *id,
            EntityEvent::Custom(_, id, _) => *id,
        }
    }

    /// Get event type name for debugging
    pub fn event_type(&self) -> &str {
        match self {
            EntityEvent::Spawned(_) => "Spawned",
            EntityEvent::Despawned(_) => "Despawned",
            EntityEvent::ComponentAdded(_, _) => "ComponentAdded",
            EntityEvent::ComponentRemoved(_, _) => "ComponentRemoved",
            EntityEvent::Custom(name, _, _) => name,
        }
    }
}

/// Event queue for deferred event processing
pub struct EventQueue {
    events: std::collections::VecDeque<EntityEvent>,
    capacity: usize,
}

impl EventQueue {
    /// Create new event queue
    pub fn new() -> Self {
        Self {
            events: std::collections::VecDeque::new(),
            capacity: 1024,
        }
    }

    /// Create with specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: std::collections::VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Add event to queue
    pub fn push(&mut self, event: EntityEvent) {
        if self.events.len() < self.capacity {
            self.events.push_back(event);
        } else {
            eprintln!("Event queue overflow! Capacity: {}", self.capacity);
        }
    }

    /// Get next event
    pub fn pop(&mut self) -> Option<EntityEvent> {
        self.events.pop_front()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get number of pending events
    pub fn len(&self) -> usize {
        self.events.len()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[derive(Clone, Copy, Debug)]
    struct TestComponent;

    #[test]
    fn test_event_queue_push_pop() {
        let mut queue = EventQueue::new();
        let mut world = World::new();
        let id = world.spawn((TestComponent,));

        queue.push(EntityEvent::Spawned(id));
        assert!(!queue.is_empty());

        let event = queue.pop();
        assert!(event.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_event_entity_id() {
        let mut world = World::new();
        let id = world.spawn((TestComponent,));
        let event = EntityEvent::Spawned(id);
        assert_eq!(event.entity_id(), id);
    }
}
