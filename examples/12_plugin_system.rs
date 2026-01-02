//! Example 12: Plugin System with Names
//! 
//! This example demonstrates the enhanced plugin system with names:
//! - Plugin identification and logging
//! - Plugin introspection capabilities
//! - Better debugging and error messages

use archetype_ecs::{App, Plugin, World, System, SystemAccess};

// Example plugin for rendering systems
struct RenderPlugin;
impl Plugin for RenderPlugin {
    fn plugin_name(&self) -> &'static str {
        "RenderPlugin"
    }
    
    fn build(&self, app: &mut App) {
        println!("  🎨 Setting up {}...", self.plugin_name());
        // In a real plugin, you would add render systems, resources, etc.
        app.add_system(Box::new(RenderSystem));
        println!("  ✅ {} configured!", self.plugin_name());
    }
}

// Example plugin for physics systems
struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn plugin_name(&self) -> &'static str {
        "PhysicsPlugin"
    }
    
    fn build(&self, app: &mut App) {
        println!("  ⚛️  Setting up {}...", self.plugin_name());
        // In a real plugin, you would add physics systems, resources, etc.
        app.add_system(Box::new(PhysicsSystem));
        println!("  ✅ {} configured!", self.plugin_name());
    }
}

// Example plugin for input systems
struct InputPlugin;
impl Plugin for InputPlugin {
    fn plugin_name(&self) -> &'static str {
        "InputPlugin"
    }
    
    fn build(&self, app: &mut App) {
        println!("  🎮 Setting up {}...", self.plugin_name());
        // In a real plugin, you would add input systems, resources, etc.
        app.add_system(Box::new(InputSystem));
        println!("  ✅ {} configured!", self.plugin_name());
    }
}

// Example systems for demonstration
struct RenderSystem;
impl System for RenderSystem {
    fn access(&self) -> SystemAccess {
        SystemAccess::empty()
    }
    
    fn name(&self) -> &'static str {
        "RenderSystem"
    }
    
    fn run(&mut self, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  🖼️  Rendering frame...");
        Ok(())
    }
}

struct PhysicsSystem;
impl System for PhysicsSystem {
    fn access(&self) -> SystemAccess {
        SystemAccess::empty()
    }
    
    fn name(&self) -> &'static str {
        "PhysicsSystem"
    }
    
    fn run(&mut self, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  ⚛️  Updating physics...");
        Ok(())
    }
}

struct InputSystem;
impl System for InputSystem {
    fn access(&self) -> SystemAccess {
        SystemAccess::empty()
    }
    
    fn name(&self) -> &'static str {
        "InputSystem"
    }
    
    fn run(&mut self, _world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        println!("  🎮 Processing input...");
        Ok(())
    }
}

fn main() {
    println!("=== Plugin System with Names Example ===\n");
    
    // Create app and add plugins
    println!("Creating application with plugins...\n");
    
    let mut app = App::new();
    
    // Add plugins - notice the logging with plugin names
    app.add_plugin(RenderPlugin);
    app.add_plugin(PhysicsPlugin);
    app.add_plugin(InputPlugin);
    
    println!("\n=== Running Application ===");
    
    // Run a few frames to demonstrate the systems
    for frame in 1..=3 {
        println!("\n--- Frame {} ---", frame);
        if let Err(e) = app.update() {
            println!("Error running frame {}: {:?}", frame, e);
            break;
        }
    }
    
    println!("\n=== Key Benefits of Plugin Names ===");
    println!("✅ Better logging: Each plugin registration is clearly identified");
    println!("✅ Debugging: Easy to track which plugin is doing what");
    println!("✅ Error messages: More informative when something goes wrong");
    println!("✅ Profiling: Can measure plugin-specific performance");
    println!("✅ Documentation: Self-documenting code structure");
    
    println!("\n=== Example Complete ===");
}
