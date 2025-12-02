use crate::entity::EntityId;
use crate::world::World;

/// World inspector for debugging
pub struct WorldInspector;

impl WorldInspector {
    /// Get total entity count
    pub fn entity_count(world: &World) -> usize {
        world.entity_count() as usize
    }

    /// Get archetype summary
    pub fn archetype_summary(world: &World) -> Vec<ArchetypeInfo> {
        let mut infos = Vec::new();

        for (id, archetype) in world.archetypes().iter().enumerate() {
            let signature: Vec<String> = archetype
                .signature()
                .iter()
                .map(|type_id| format!("{type_id:?}"))
                .collect();

            infos.push(ArchetypeInfo {
                id,
                signature,
                entity_count: archetype.len(),
                component_count: archetype.signature().len(),
            });
        }

        infos
    }

    /// Print world summary to console
    pub fn print_summary(world: &World) {
        println!("=== World Summary ===");
        println!("Entities: {}", Self::entity_count(world));
        println!("Archetypes: {}", world.archetype_count());

        println!("\n=== Archetypes ===");
        for info in Self::archetype_summary(world) {
            println!(
                "Archetype {}: {} entities, {} components",
                info.id, info.entity_count, info.component_count
            );
        }
    }

    /// Print entity details
    pub fn print_entity(world: &World, entity: EntityId) {
        if let Some(location) = world.get_entity_location(entity) {
            println!("=== Entity {entity:?} ===");
            println!("Archetype: {}", location.archetype_id);
            println!("Row: {}", location.archetype_row);

            if let Some(archetype) = world.archetypes().get(location.archetype_id) {
                println!("Components: {} types", archetype.signature().len());
            }
        } else {
            println!("Entity {entity:?} not found");
        }
    }
}

/// Archetype information for debugging
#[derive(Clone, Debug)]
pub struct ArchetypeInfo {
    pub id: usize,
    pub signature: Vec<String>,
    pub entity_count: usize,
    pub component_count: usize,
}

use std::collections::VecDeque;

/// Performance diagnostics
#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    frame_times: VecDeque<f32>,
    max_samples: usize,
}

impl Diagnostics {
    /// Create new diagnostics tracker
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::new(),
            max_samples: 60,
        }
    }

    /// Record a frame time in milliseconds
    pub fn record_frame_time(&mut self, time_ms: f32) {
        self.frame_times.push_back(time_ms);
        if self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }
    }

    /// Get average FPS
    pub fn fps(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let avg_ms: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        if avg_ms > 0.0 {
            1000.0 / avg_ms
        } else {
            0.0
        }
    }

    /// Get average frame time in milliseconds
    pub fn avg_frame_time(&self) -> f32 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
    }

    /// Get min frame time
    pub fn min_frame_time(&self) -> f32 {
        self.frame_times
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
    }

    /// Get max frame time
    pub fn max_frame_time(&self) -> f32 {
        self.frame_times
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
    }

    /// Print diagnostics
    pub fn print(&self) {
        println!("=== Diagnostics ===");
        println!("FPS: {:.1}", self.fps());
        println!("Avg Frame Time: {:.2}ms", self.avg_frame_time());
        println!("Min Frame Time: {:.2}ms", self.min_frame_time());
        println!("Max Frame Time: {:.2}ms", self.max_frame_time());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics() {
        let mut diag = Diagnostics::new();

        // Record some frame times (16.67ms = 60 FPS)
        for _ in 0..10 {
            diag.record_frame_time(16.67);
        }

        assert!((diag.fps() - 60.0).abs() < 1.0);
        assert!((diag.avg_frame_time() - 16.67).abs() < 0.1);
    }

    #[test]
    fn test_world_inspector() {
        let world = World::new();
        assert_eq!(WorldInspector::entity_count(&world), 0);
    }
}
