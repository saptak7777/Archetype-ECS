use crate::error::Result;
use crate::event_bus::{Event, EventSubscriber};
use std::any::TypeId;
use std::sync::{Arc, Mutex};

/// Logging subscriber that prints all events
pub struct LoggingSubscriber;

impl EventSubscriber for LoggingSubscriber {
    fn on_event(&mut self, event: &dyn Event) -> Result<()> {
        println!("Event: {}", event.event_name());
        Ok(())
    }

    fn name(&self) -> &str {
        "LoggingSubscriber"
    }
}

/// Statistics subscriber that counts events
pub struct StatisticsSubscriber {
    pub event_count: usize,
    pub event_types: std::collections::HashMap<TypeId, usize>,
}

impl StatisticsSubscriber {
    pub fn new() -> Self {
        Self {
            event_count: 0,
            event_types: std::collections::HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.event_count = 0;
        self.event_types.clear();
    }
}

impl EventSubscriber for StatisticsSubscriber {
    fn on_event(&mut self, event: &dyn Event) -> Result<()> {
        self.event_count += 1;
        *self.event_types.entry(event.event_type_id()).or_insert(0) += 1;
        Ok(())
    }

    fn name(&self) -> &str {
        "StatisticsSubscriber"
    }
}

impl Default for StatisticsSubscriber {
    fn default() -> Self {
        Self::new()
    }
}

type EventCallback = Box<dyn Fn(&dyn Event) -> Result<()> + Send + Sync>;
type EventFilter = Box<dyn Fn(&dyn Event) -> bool + Send + Sync>;

/// Callback-based subscriber
pub struct CallbackSubscriber {
    callback: Arc<Mutex<EventCallback>>,
}

impl CallbackSubscriber {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(&dyn Event) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            callback: Arc::new(Mutex::new(Box::new(callback))),
        }
    }
}

impl EventSubscriber for CallbackSubscriber {
    fn on_event(&mut self, event: &dyn Event) -> Result<()> {
        let callback = self.callback.lock().unwrap();
        callback(event)
    }

    fn name(&self) -> &str {
        "CallbackSubscriber"
    }
}

/// Filter-based subscriber (only processes matching events)
pub struct FilteredSubscriber {
    filter: Arc<EventFilter>,
    handler: Arc<Mutex<EventCallback>>,
}

impl FilteredSubscriber {
    pub fn new<F, H>(filter: F, handler: H) -> Self
    where
        F: Fn(&dyn Event) -> bool + Send + Sync + 'static,
        H: Fn(&dyn Event) -> Result<()> + Send + Sync + 'static,
    {
        Self {
            filter: Arc::new(Box::new(filter)),
            handler: Arc::new(Mutex::new(Box::new(handler))),
        }
    }
}

impl EventSubscriber for FilteredSubscriber {
    fn on_event(&mut self, event: &dyn Event) -> Result<()> {
        if (self.filter)(event) {
            let handler = self.handler.lock().unwrap();
            handler(event)?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "FilteredSubscriber"
    }
}
