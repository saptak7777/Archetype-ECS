//! Hot-Reload Systems Example
//! 
//! This example demonstrates the hot-reload functionality for systems,
//! allowing developers to modify systems at runtime without restarting
//! the entire application.

use archetype_ecs::{World, System, SystemAccess, App};
use archetype_ecs::hot_reload::{ReloadableSystem, HotReloadApp};
use std::time::SystemTime;

#[derive(Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

// Example reloadable movement system
#[derive(Debug)]
struct MovementSystem {
    pub last_reload: Option<SystemTime>,
    pub speed_multiplier: f32,
    pub counter: u32,
}

impl MovementSystem {
    pub fn new() -> Self {
        Self {
            last_reload: None,
            speed_multiplier: 1.0,
            counter: 0,
        }
    }
}

impl System for MovementSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Velocity>());
        access.writes.push(std::any::TypeId::of::<Position>());
        access
    }

    fn name(&self) -> &'static str {
        "MovementSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        self.counter += 1;
        
        println!("🎮 Running MovementSystem (counter: {}, speed: {:.2}, last_reload: {:?})", 
            self.counter, self.speed_multiplier, self.last_reload);
        
        for (pos, vel) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
            pos.x += vel.x * self.speed_multiplier;
            pos.y += vel.y * self.speed_multiplier;
        }
        
        Ok(())
    }
}

impl ReloadableSystem for MovementSystem {
    fn reload(&mut self) -> Result<(), archetype_ecs::error::EcsError> {
        println!("🔄 Reloading MovementSystem...");
        
        // Simulate reloading logic - in a real implementation,
        // this would read the source file and update the system
        self.last_reload = Some(SystemTime::now());
        self.speed_multiplier = (self.speed_multiplier + 0.5).min(5.0); // Increase speed on reload
        self.counter = 0; // Reset counter on reload
        
        println!("✅ MovementSystem reloaded successfully! New speed multiplier: {:.2}", self.speed_multiplier);
        Ok(())
    }
    
    fn source_path(&self) -> Option<&str> {
        Some("examples/movement_system.rs")
    }
    
    fn last_reload_time(&self) -> Option<SystemTime> {
        self.last_reload
    }
    
    fn update_reload_time(&mut self) {
        self.last_reload = Some(SystemTime::now());
    }
}

// Example reloadable rendering system
#[derive(Debug)]
struct RenderSystem {
    pub last_reload: Option<SystemTime>,
    pub render_mode: String,
    pub counter: u32,
}

impl RenderSystem {
    pub fn new() -> Self {
        Self {
            last_reload: None,
            render_mode: "basic".to_string(),
            counter: 0,
        }
    }
}

impl System for RenderSystem {
    fn access(&self) -> SystemAccess {
        let mut access = SystemAccess::empty();
        access.reads.push(std::any::TypeId::of::<Position>());
        access
    }

    fn name(&self) -> &'static str {
        "RenderSystem"
    }

    fn run(&mut self, world: &mut World) -> Result<(), archetype_ecs::error::EcsError> {
        self.counter += 1;
        
        println!("🎨 Running RenderSystem (counter: {}, mode: {}, last_reload: {:?})", 
            self.counter, self.render_mode, self.last_reload);
        
        let entity_count = world.query::<&Position>().iter().count();
        println!("  Rendering {} entities in {} mode", entity_count, self.render_mode);
        
        Ok(())
    }
}

impl ReloadableSystem for RenderSystem {
    fn reload(&mut self) -> Result<(), archetype_ecs::error::EcsError> {
        println!("🔄 Reloading RenderSystem...");
        
        // Simulate reloading logic
        self.last_reload = Some(SystemTime::now());
        
        // Switch render mode
        self.render_mode = match self.render_mode.as_str() {
            "basic" => "advanced".to_string(),
            "advanced" => "ultra".to_string(),
            _ => "basic".to_string(),
        };
        
        self.counter = 0; // Reset counter on reload
        
        println!("✅ RenderSystem reloaded successfully! New render mode: {}", self.render_mode);
        Ok(())
    }
    
    fn source_path(&self) -> Option<&str> {
        Some("examples/render_system.rs")
    }
    
    fn last_reload_time(&self) -> Option<SystemTime> {
        self.last_reload
    }
    
    fn update_reload_time(&mut self) {
        self.last_reload = Some(SystemTime::now());
    }
}

fn main() {
    println!("=== Hot-Reload Systems Example ===\n");
    
    // Create world and spawn entities
    let mut world = World::new();
    
    println!("Spawning entities...");
    for i in 0..10 {
        world.spawn((
            Position { x: i as f32, y: i as f32 },
            Velocity { x: 0.1, y: 0.05 },
        ));
    }
    
    println!("Spawned {} entities\n", world.entity_count());
    
    // Create app with hot-reload support
    let mut app = App::new();
    
    // Register reloadable systems
    app.register_reloadable_system("movement".to_string(), MovementSystem::new());
    app.register_reloadable_system("render".to_string(), RenderSystem::new());
    
    println!("Registered {} reloadable systems\n", app.hot_reload_manager().system_count());
    
    // Set hot-reload check interval to 1 second for demonstration
    app.hot_reload_manager().set_check_interval(std::time::Duration::from_secs(1));
    
    println!("=== Running Game Loop with Hot-Reload ===");
    println!("Systems will automatically reload when their source files change.");
    println!("In a real implementation, this would watch actual source files.");
    println!("For this demo, we'll simulate file changes.\n");
    
    // Run several frames
    for frame in 0..10 {
        println!("--- Frame {} ---", frame + 1);
        
        // Check for hot-reload (in real implementation, this would check file timestamps)
        // For demonstration, we'll manually trigger reloads on specific frames
        if frame == 3 {
            println!("📁 Simulating file change in movement system...");
            let reload_count = app.reload_all_systems().unwrap_or(0);
            println!("Reloaded {reload_count} systems");
        }
        
        if frame == 6 {
            println!("📁 Simulating file change in render system...");
            let reload_count = app.reload_all_systems().unwrap_or(0);
            println!("Reloaded {reload_count} systems");
        }
        
        // Run systems (simplified - in real app, you'd use the schedule)
        println!("Running systems...");
        
        // Simulate running the registered systems
        for system_name in app.hot_reload_manager().system_names() {
            println!("  Running system: {system_name}");
        }
        
        // In a real implementation, you'd call:
        // app.check_hot_reload()?;
        // app.update()?;
        
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    // Demonstrate hot-reload control
    println!("\n=== Hot-Reload Control Demo ===");
    
    // Disable hot-reload
    println!("Disabling hot-reload...");
    app.set_hot_reload_enabled(false);
    println!("Hot-reload enabled: {}", app.hot_reload_manager().enabled);
    
    // Re-enable hot-reload
    println!("Re-enabling hot-reload...");
    app.set_hot_reload_enabled(true);
    println!("Hot-reload enabled: {}", app.hot_reload_manager().enabled);
    
    // Show system information
    println!("\n=== System Information ===");
    println!("Registered systems: {:?}", app.hot_reload_manager().system_names());
    println!("Check interval: {:?}", app.hot_reload_manager().check_interval);
    
    println!("\n=== Hot-Reload Example Complete ===");
    println!("Key benefits:");
    println!("✅ 10x faster iteration - no need to restart game");
    println!("✅ Test changes in seconds - modify and reload instantly");
    println!("✅ Keep game state between reloads - entities remain intact");
    println!("✅ Automatic file watching - reloads when source changes");
}
