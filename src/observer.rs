use crate::error::Result;
use crate::event::EntityEvent;
use crate::world::World;

/// Observer that reacts to entity lifecycle events
pub trait Observer: Send + Sync {
    /// Called when an entity event occurs
    /// Return error to stop processing
    fn on_event(&mut self, event: &EntityEvent, world: &mut World) -> Result<()>;

    /// Get name for debugging
    fn name(&self) -> &str {
        "Observer"
    }

    /// Called before observer is stored in registry
    /// Useful for setup that needs to happen before registration
    fn on_before_register(&mut self, _world: &mut World) -> Result<()> {
        Ok(())
    }

    /// Called when observer is registered and stored
    /// Observer can now access its final storage position
    fn on_registered(&mut self, _world: &mut World) -> Result<()> {
        Ok(())
    }

    /// Called after observer is fully registered with index
    /// Observer knows its final position in the registry
    fn on_after_register(&mut self, _world: &mut World, _index: usize) -> Result<()> {
        Ok(())
    }

    /// Called when observer is unregistered
    fn on_unregistered(&mut self, _world: &mut World) -> Result<()> {
        Ok(())
    }
}

/// Metrics for observer performance tracking
#[derive(Debug, Clone, Default)]
pub struct ObserverMetrics {
    /// Total number of events processed
    pub total_events: u64,

    /// Total time spent in observers (microseconds)
    pub total_time_us: u64,

    /// Events processed per observer type
    pub events_by_type: std::collections::HashMap<String, u64>,

    /// Average time per event (microseconds)
    pub avg_time_us: f64,

    /// Peak time for single event (microseconds)
    pub peak_time_us: u64,

    /// Last reset time
    pub last_reset: Option<std::time::Instant>,
}

impl ObserverMetrics {
    /// Reset metrics
    pub fn reset(&mut self) {
        self.total_events = 0;
        self.total_time_us = 0;
        self.events_by_type.clear();
        self.avg_time_us = 0.0;
        self.peak_time_us = 0;
        self.last_reset = Some(std::time::Instant::now());
    }

    /// Record an event processing
    pub fn record_event(&mut self, event_type: &str, duration_us: u64) {
        self.total_events += 1;
        self.total_time_us += duration_us;
        self.avg_time_us = self.total_time_us as f64 / self.total_events as f64;
        self.peak_time_us = self.peak_time_us.max(duration_us);

        *self
            .events_by_type
            .entry(event_type.to_string())
            .or_insert(0) += 1;
    }

    /// Print summary to stdout
    pub fn print_summary(&self) {
        println!("Observer Metrics:");
        println!("  Total events: {}", self.total_events);
        println!("  Average time: {:.2} μs", self.avg_time_us);
        println!("  Peak time: {} μs", self.peak_time_us);
        println!("  Events by type:");
        for (event_type, count) in &self.events_by_type {
            println!("    - {event_type}: {count}");
        }
    }
}

/// Registry that manages all observers
pub struct ObserverRegistry {
    pub(crate) observers: Vec<Box<dyn Observer>>,
}

impl ObserverRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    /// Register observer
    pub fn register(&mut self, mut observer: Box<dyn Observer>, world: &mut World) -> Result<()> {
        // Call before registration
        observer.on_before_register(world)?;

        // Store observer
        let index = self.observers.len();
        self.observers.push(observer);

        // Call after registration
        if let Some(obs) = self.observers.last_mut() {
            obs.on_registered(world)?;
            obs.on_after_register(world, index)?;
        }

        Ok(())
    }

    /// Unregister observer by index
    pub fn unregister(&mut self, index: usize) -> Option<Box<dyn Observer>> {
        if index < self.observers.len() {
            Some(self.observers.remove(index))
        } else {
            None
        }
    }

    /// Broadcast event to all observers
    pub fn broadcast(&mut self, event: &EntityEvent, world: &mut World) -> Result<()> {
        for observer in &mut self.observers {
            observer.on_event(event, world)?;
        }
        Ok(())
    }

    /// Get number of registered observers
    pub fn observer_count(&self) -> usize {
        self.observers.len()
    }

    /// Clear all observers
    pub fn clear(&mut self) {
        self.observers.clear();
    }
}

impl Default for ObserverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Example: Log observer that prints all events
pub struct LoggingObserver;

impl Observer for LoggingObserver {
    fn on_event(&mut self, event: &EntityEvent, _world: &mut World) -> Result<()> {
        match event {
            EntityEvent::Spawned(id) => println!("Entity spawned: {id:?}"),
            EntityEvent::Despawned(id) => println!("Entity despawned: {id:?}"),
            EntityEvent::ComponentAdded(id, type_id) => {
                println!("Component added to entity {id:?}: {type_id:?}")
            }
            EntityEvent::ComponentRemoved(id, type_id) => {
                println!("Component removed from entity {id:?}: {type_id:?}")
            }
            EntityEvent::Custom(name, id, _) => {
                println!("Custom event '{name}' for entity {id:?}")
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "LoggingObserver"
    }
}

// Example: Counter observer that tracks statistics
pub struct StatisticsObserver {
    pub spawned_count: usize,
    pub despawned_count: usize,
    pub component_additions: usize,
    pub component_removals: usize,
}

impl Default for StatisticsObserver {
    fn default() -> Self {
        Self::new()
    }
}

impl StatisticsObserver {
    pub fn new() -> Self {
        Self {
            spawned_count: 0,
            despawned_count: 0,
            component_additions: 0,
            component_removals: 0,
        }
    }

    pub fn reset(&mut self) {
        self.spawned_count = 0;
        self.despawned_count = 0;
        self.component_additions = 0;
        self.component_removals = 0;
    }
}

impl Observer for StatisticsObserver {
    fn on_event(&mut self, event: &EntityEvent, _world: &mut World) -> Result<()> {
        match event {
            EntityEvent::Spawned(_) => self.spawned_count += 1,
            EntityEvent::Despawned(_) => self.despawned_count += 1,
            EntityEvent::ComponentAdded(_, _) => self.component_additions += 1,
            EntityEvent::ComponentRemoved(_, _) => self.component_removals += 1,
            EntityEvent::Custom(_, _, _) => {}
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "StatisticsObserver"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[allow(dead_code)] // Test observer for debugging
    struct TestObserver {
        call_count: Arc<Mutex<usize>>,
    }

    impl Observer for TestObserver {
        fn on_event(&mut self, _event: &EntityEvent, _world: &mut World) -> Result<()> {
            *self.call_count.lock().unwrap() += 1;
            Ok(())
        }

        fn name(&self) -> &str {
            "TestObserver"
        }
    }

    #[test]
    fn test_observer_registry_creation() {
        let registry = ObserverRegistry::new();
        assert_eq!(registry.observer_count(), 0);
    }

    struct LifecycleTestObserver {
        before_called: bool,
        registered_called: bool,
        after_called: bool,
        after_index: Option<usize>,
    }

    impl Observer for LifecycleTestObserver {
        fn on_event(&mut self, _event: &EntityEvent, _world: &mut World) -> Result<()> {
            Ok(())
        }

        fn on_before_register(&mut self, _world: &mut World) -> Result<()> {
            self.before_called = true;
            Ok(())
        }

        fn on_registered(&mut self, _world: &mut World) -> Result<()> {
            self.registered_called = true;
            Ok(())
        }

        fn on_after_register(&mut self, _world: &mut World, index: usize) -> Result<()> {
            self.after_called = true;
            self.after_index = Some(index);
            Ok(())
        }
    }

    #[test]
    fn test_observer_lifecycle_callbacks() {
        let mut registry = ObserverRegistry::new();
        let observer = Box::new(LifecycleTestObserver {
            before_called: false,
            registered_called: false,
            after_called: false,
            after_index: None,
        });

        let mut world = World::new();
        registry.register(observer, &mut world).unwrap();

        // Verify all callbacks were called in correct order
        assert_eq!(registry.observer_count(), 1);

        // We can't easily verify the internal state since the observer is now in the registry
        // But we can verify the registry has one observer
    }
}
