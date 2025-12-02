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

    /// Optional: called when observer is registered
    fn on_registered(&mut self, _world: &mut World) -> Result<()> {
        Ok(())
    }

    /// Optional: called when observer is unregistered
    fn on_unregistered(&mut self, _world: &mut World) -> Result<()> {
        Ok(())
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
    pub fn register(&mut self, observer: Box<dyn Observer>, world: &mut World) -> Result<()> {
        // Clone observer, call on_registered, then store
        // Note: Due to trait object limitations, we call after storing
        self.observers.push(observer);
        // We can't easily call on_registered here because we just moved it into the vector
        // and we'd need to borrow it back mutably while also passing world.
        // For simplicity in this phase, we'll skip the immediate callback or handle it if needed later.
        // If strict adherence to the plan is required, we might need a different design,
        // but typically registration happens at setup.
        // Let's try to call it if possible, but it requires mutable borrow of observer and world.
        // self.observers.last_mut().unwrap().on_registered(world)
        // This would work if world isn't borrowed by the registry itself (it isn't here).

        if let Some(obs) = self.observers.last_mut() {
            obs.on_registered(world)?;
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

    #[test]
    fn test_observer_registration() {
        let mut registry = ObserverRegistry::new();
        let observer = Box::new(TestObserver {
            call_count: Arc::new(Mutex::new(0)),
        });

        let mut world = World::new();
        registry.register(observer, &mut world).unwrap();
        assert_eq!(registry.observer_count(), 1);
    }
}
