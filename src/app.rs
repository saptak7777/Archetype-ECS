use crate::error::Result;
use crate::executor::Executor;
use crate::plugin::Plugin;
use crate::schedule::Schedule;
use crate::system::BoxedSystem;
use crate::world::World;

/// Main application entry point
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    pub executor: Executor,
}

impl App {
    /// Create new application
    pub fn new() -> Self {
        let schedule = Schedule::new();
        Self {
            world: World::new(),
            executor: Executor::new(Schedule::new()),
            schedule,
        }
    }

    /// Add a plugin
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Add a system
    pub fn add_system(&mut self, system: BoxedSystem) -> &mut Self {
        self.schedule.add_system(system);
        self
    }

    /// Run the application (one frame)
    pub fn update(&mut self) -> Result<()> {
        // Sync schedule to executor if needed
        // For now, we just recreate executor with current schedule
        // In a real engine, we'd have a better way to update the executor
        // or the executor would hold a reference to the schedule

        // Note: This is a simplification. Ideally Executor holds the schedule.
        // But Schedule is moved into Executor in current design.
        // We need to refactor Executor to take &Schedule or clone it.
        // For now, let's just rebuild Executor for this frame

        // Actually, let's fix the design slightly.
        // We'll keep schedule in App and pass it to Executor or have Executor hold it.
        // The current Executor::new takes Schedule by value.

        // Let's clone the schedule for execution since Schedule is cloneable (if systems are?)
        // Systems are Box<dyn System>, which isn't Clone.
        // So we can't clone Schedule easily.

        // Alternative: App holds Executor, and we add systems directly to Executor's schedule?
        // Or we build the schedule in App and then move it to Executor?

        // Let's assume we build everything in App.schedule, then when we run, we might need to
        // move it to executor or have executor work on it.

        // For this iteration, let's make Executor take &mut Schedule.
        // But Executor::execute_frame takes &mut self (which has schedule).

        // Let's change Executor to be created with the Schedule when we start running?
        // Or just expose executor.schedule.

        self.executor.schedule = std::mem::take(&mut self.schedule);
        self.executor.execute_frame(&mut self.world)?;
        self.schedule = std::mem::take(&mut self.executor.schedule);

        Ok(())
    }

    /// Run the application loop (simplified)
    pub fn run(&mut self) -> Result<()> {
        loop {
            self.update()?;
            // Break condition?
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn build(&self, _app: &mut App) {
            // Do nothing
        }
    }

    #[test]
    fn test_app_creation() {
        let mut app = App::new();
        app.add_plugin(TestPlugin);
    }
}
