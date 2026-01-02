use crate::error::Result;
use crate::executor::Executor;
use crate::hot_reload::{HotReloadManager, HotReloadApp, ReloadableSystem};
use crate::plugin::Plugin;
use crate::schedule::Schedule;
use crate::system::BoxedSystem;
use crate::world::World;

/// Main application entry point
pub struct App {
    pub world: World,
    pub schedule: Schedule,
    hot_reload_manager: HotReloadManager,
}

impl App {
    /// Create new application
    pub fn new() -> Self {
        Self {
            world: World::new(),
            schedule: Schedule::new(),
            hot_reload_manager: HotReloadManager::new(),
        }
    }

    /// Add a plugin
    pub fn add_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        println!("📦 Registering plugin: {}", plugin.plugin_name());
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
        // Create executor with schedule reference for this frame
        let mut executor = Executor::new(&mut self.schedule);
        executor.execute_frame(&mut self.world)?;
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

impl HotReloadApp for App {
    fn hot_reload_manager(&mut self) -> &mut HotReloadManager {
        &mut self.hot_reload_manager
    }
    
    fn register_reloadable_system<S: ReloadableSystem + 'static>(&mut self, name: String, system: S) {
        self.hot_reload_manager.register_system(name, system);
    }
    
    fn check_hot_reload(&mut self) -> Result<usize> {
        self.hot_reload_manager.check_and_reload(&mut self.world)
    }
    
    fn reload_all_systems(&mut self) -> Result<usize> {
        self.hot_reload_manager.reload_all(&mut self.world)
    }
    
    fn set_hot_reload_enabled(&mut self, enabled: bool) {
        self.hot_reload_manager.set_enabled(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;
    impl Plugin for TestPlugin {
        fn plugin_name(&self) -> &'static str {
            "TestPlugin"
        }
        
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
