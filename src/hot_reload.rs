//! Hot-Reload Systems
//! 
//! This module provides hot-reload functionality for systems,
//! allowing developers to modify systems at runtime without restarting
//! the entire application.

use crate::error::Result;
use crate::system::System;
use crate::system::SystemAccess;
use crate::world::World;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Time provider abstraction for hot-reload testing
pub trait TimeProvider: Send + Sync {
    fn now(&self) -> std::time::SystemTime;
    fn file_modified(&self, path: &str) -> std::io::Result<std::time::SystemTime>;
}

/// System time provider for production use
pub struct SystemTimeProvider;
impl TimeProvider for SystemTimeProvider {
    fn now(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
    
    fn file_modified(&self, path: &str) -> std::io::Result<std::time::SystemTime> {
        std::fs::metadata(path)?.modified()
    }
}

/// Trait for systems that support hot-reloading
pub trait ReloadableSystem: System {
    /// Reload the system's logic
    fn reload(&mut self) -> Result<()>;
    
    /// Get the source file path for this system
    fn source_path(&self) -> Option<&str>;
    
    /// Check if the source file has been modified
    fn is_modified(&self) -> bool {
        if let Some(path) = self.source_path() {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    return self.last_reload_time().is_none_or(|last| modified > last);
                }
            }
        }
        false
    }
    
    /// Get the last time this system was reloaded
    fn last_reload_time(&self) -> Option<std::time::SystemTime>;
    
    /// Update the last reload time
    fn update_reload_time(&mut self);
}

/// Hot-reload manager for tracking and reloading systems
pub struct HotReloadManager {
    /// Map of system names to their reloadable instances
    pub systems: HashMap<String, Box<dyn ReloadableSystem>>,
    /// Last check time for file modifications
    pub last_check: Instant,
    /// Check interval in milliseconds
    pub check_interval: Duration,
    /// Whether hot-reload is enabled
    pub enabled: bool,
    /// Time provider for testing isolation
    #[allow(dead_code)] // Used for testing isolation
    time_provider: Box<dyn TimeProvider>,
}

impl HotReloadManager {
    /// Create a new hot-reload manager
    pub fn new() -> Self {
        Self::new_with_provider(Box::new(SystemTimeProvider))
    }
    
    /// Create a new hot-reload manager with custom time provider
    pub fn new_with_provider(provider: Box<dyn TimeProvider>) -> Self {
        Self {
            systems: HashMap::new(),
            last_check: Instant::now(),
            check_interval: Duration::from_millis(500), // Check every 500ms
            enabled: true,
            time_provider: provider,
        }
    }
    
    /// Set the check interval for file modifications
    pub fn set_check_interval(&mut self, interval: Duration) {
        self.check_interval = interval;
    }
    
    /// Enable or disable hot-reload
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Register a reloadable system
    pub fn register_system<S: ReloadableSystem + 'static>(&mut self, name: String, system: S) {
        self.systems.insert(name, Box::new(system));
    }
    
    /// Check for modified systems and reload them
    pub fn check_and_reload(&mut self, _world: &mut World) -> Result<usize> {
        if !self.enabled {
            return Ok(0);
        }
        
        let now = Instant::now();
        if now.duration_since(self.last_check) < self.check_interval {
            return Ok(0);
        }
        
        let mut reloaded_count = 0;
        let mut systems_to_reload = Vec::new();
        
        // Check each system for modifications
        for (name, system) in &self.systems {
            if system.is_modified() {
                systems_to_reload.push(name.clone());
            }
        }
        
        // Reload modified systems
        for name in systems_to_reload {
            if let Some(system) = self.systems.get_mut(&name) {
                println!("🔄 Hot-reloading system: {name}");
                
                match system.reload() {
                    Ok(()) => {
                        system.update_reload_time();
                        reloaded_count += 1;
                        println!("✅ Successfully reloaded system: {name}");
                    }
                    Err(e) => {
                        println!("❌ Failed to reload system {name}: {e:?}");
                    }
                }
            }
        }
        
        self.last_check = now;
        Ok(reloaded_count)
    }
    
    /// Force reload all registered systems
    pub fn reload_all(&mut self, _world: &mut World) -> Result<usize> {
        if !self.enabled {
            return Ok(0);
        }
        
        let mut reloaded_count = 0;
        
        for (name, system) in &mut self.systems {
            println!("🔄 Force reloading system: {name}");
            
            match system.reload() {
                Ok(()) => {
                    system.update_reload_time();
                    reloaded_count += 1;
                    println!("✅ Successfully reloaded system: {name}");
                }
                Err(e) => {
                    println!("❌ Failed to reload system {name}: {e:?}");
                }
            }
        }
        
        Ok(reloaded_count)
    }
    
    /// Get the number of registered systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
    
    /// Get the names of all registered systems
    pub fn system_names(&self) -> Vec<&String> {
        self.systems.keys().collect()
    }
    
    /// Remove a system from hot-reload tracking
    pub fn unregister_system(&mut self, name: &str) -> Option<Box<dyn ReloadableSystem>> {
        self.systems.remove(name)
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for App to support hot-reload
pub trait HotReloadApp {
    /// Get mutable reference to the hot-reload manager
    fn hot_reload_manager(&mut self) -> &mut HotReloadManager;
    
    /// Register a reloadable system
    fn register_reloadable_system<S: ReloadableSystem + 'static>(&mut self, name: String, system: S);
    
    /// Check for and reload modified systems
    fn check_hot_reload(&mut self) -> Result<usize>;
    
    /// Force reload all systems
    fn reload_all_systems(&mut self) -> Result<usize>;
    
    /// Enable or disable hot-reload
    fn set_hot_reload_enabled(&mut self, enabled: bool);
}

/// Macro to create a reloadable system with file watching
#[macro_export]
macro_rules! reloadable_system {
    ($system_type:ty, $file_path:expr) => {
        impl $crate::hot_reload::ReloadableSystem for $system_type {
            fn reload(&mut self) -> $crate::error::Result<()> {
                // In a real implementation, this would:
                // 1. Read the source file
                // 2. Parse the new system implementation
                // 3. Update the current system with new logic
                println!("Reloading system from: {}", $file_path);
                
                // For demonstration, we'll just print a message
                // In practice, you'd use dynamic loading or code generation
                Ok(())
            }
            
            fn source_path(&self) -> Option<&str> {
                Some($file_path)
            }
            
            fn last_reload_time(&self) -> Option<std::time::SystemTime> {
                // Store and return the last reload time
                // This would be a field in the actual system
                None
            }
            
            fn update_reload_time(&mut self) {
                // Update the last reload time
                // This would update a field in the actual system
            }
        }
    };
}

/// Example reloadable system
#[derive(Debug)]
pub struct ExampleReloadableSystem {
    pub last_reload: Option<std::time::SystemTime>,
    pub counter: u32,
}

impl Default for ExampleReloadableSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ExampleReloadableSystem {
    pub fn new() -> Self {
        Self {
            last_reload: None,
            counter: 0,
        }
    }
}

impl System for ExampleReloadableSystem {
    fn access(&self) -> SystemAccess {
        SystemAccess::empty()
    }

    fn name(&self) -> &'static str {
        "ExampleReloadableSystem"
    }

    fn run(&mut self, _world: &mut World) -> Result<()> {
        self.counter += 1;
        println!("Running ExampleReloadableSystem (counter: {}, last_reload: {:?})", 
            self.counter, self.last_reload);
        Ok(())
    }
}

impl ReloadableSystem for ExampleReloadableSystem {
    fn reload(&mut self) -> Result<()> {
        println!("🔄 Reloading ExampleReloadableSystem...");
        
        // Simulate reloading logic
        self.last_reload = Some(std::time::SystemTime::now());
        self.counter = 0; // Reset counter on reload
        
        println!("✅ ExampleReloadableSystem reloaded successfully!");
        Ok(())
    }
    
    fn source_path(&self) -> Option<&str> {
        Some("examples/example_system.rs")
    }
    
    fn last_reload_time(&self) -> Option<std::time::SystemTime> {
        self.last_reload
    }
    
    fn update_reload_time(&mut self) {
        self.last_reload = Some(std::time::SystemTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hot_reload_manager() {
        let mut manager = HotReloadManager::new();
        let system = ExampleReloadableSystem::new();
        
        manager.register_system("test_system".to_string(), system);
        assert_eq!(manager.system_count(), 1);
        
        let names = manager.system_names();
        assert!(names.contains(&&"test_system".to_string()));
    }
    
    #[test]
    fn test_reloadable_system() {
        let mut system = ExampleReloadableSystem::new();
        
        assert!(system.source_path().is_some());
        assert!(system.last_reload_time().is_none());
        
        system.reload().expect("Failed to reload");
        assert!(system.last_reload_time().is_some());
    }
}
