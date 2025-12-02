use crate::system::SystemAccess;

/// Represents execution stages where all systems in a stage can run in parallel
#[derive(Clone, Debug)]
pub struct ExecutionStage {
    pub system_indices: Vec<usize>,
}

/// Builds execution stages from system dependencies
pub struct DependencyGraph {
    stages: Vec<ExecutionStage>,
}

impl DependencyGraph {
    /// Create graph from system accesses
    pub fn new(system_accesses: Vec<SystemAccess>) -> Self {
        let stages = Self::build_stages(&system_accesses);
        Self { stages }
    }

    /// Build execution stages (grouping parallelizable systems)
    fn build_stages(accesses: &[SystemAccess]) -> Vec<ExecutionStage> {
        if accesses.is_empty() {
            return vec![];
        }

        let mut stages = vec![];
        let mut remaining: Vec<usize> = (0..accesses.len()).collect();

        while !remaining.is_empty() {
            // Find all systems that can run in this stage
            let mut stage_systems = vec![];
            let mut next_remaining = vec![];

            for &idx in &remaining {
                let mut can_add = true;

                // Check if this system conflicts with any in current stage
                for &stage_idx in &stage_systems {
                    if accesses[idx].conflicts_with(&accesses[stage_idx]) {
                        can_add = false;
                        break;
                    }
                }

                if can_add {
                    stage_systems.push(idx);
                } else {
                    next_remaining.push(idx);
                }
            }

            if !stage_systems.is_empty() {
                stages.push(ExecutionStage {
                    system_indices: stage_systems,
                });
            }

            remaining = next_remaining;
        }

        stages
    }

    /// Get execution stages
    pub fn stages(&self) -> &[ExecutionStage] {
        &self.stages
    }

    /// Get number of stages
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Print execution plan (for debugging)
    pub fn print_schedule(&self) {
        println!("Execution Schedule ({} stages):", self.stages.len());
        for (stage_idx, stage) in self.stages.iter().enumerate() {
            println!(
                "  Stage {}: {} systems (can run in parallel)",
                stage_idx + 1,
                stage.system_indices.len()
            );
            for &sys_idx in &stage.system_indices {
                println!("    - System {sys_idx}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn test_no_conflicts_parallel() {
        let access1 = SystemAccess {
            reads: vec![TypeId::of::<i32>()],
            writes: vec![],
        };
        let access2 = SystemAccess {
            reads: vec![TypeId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access1, access2]);
        assert_eq!(graph.stage_count(), 1, "Should execute in parallel");
    }

    #[test]
    fn test_write_conflict_sequential() {
        let access1 = SystemAccess {
            reads: vec![TypeId::of::<i32>()],
            writes: vec![TypeId::of::<f32>()],
        };
        let access2 = SystemAccess {
            reads: vec![TypeId::of::<f32>()],
            writes: vec![],
        };

        let graph = DependencyGraph::new(vec![access1, access2]);
        assert_eq!(graph.stage_count(), 2, "Should execute sequentially");
    }
}
