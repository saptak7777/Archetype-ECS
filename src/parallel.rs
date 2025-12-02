use crate::dependency::{DependencyGraph, ExecutionStage};
use crate::error::Result;
use crate::system::System;
use crate::world::World;
use rayon::prelude::*;

/// Parallel executor using rayon work-stealing
pub struct ParallelExecutor {
    pub systems: Vec<Box<dyn System>>,
    dependency_graph: DependencyGraph,
}

impl ParallelExecutor {
    /// Create parallel executor from systems
    pub fn new(systems: Vec<Box<dyn System>>) -> Self {
        // Get system accesses
        let accesses: Vec<_> = systems.iter().map(|s| s.access()).collect();

        // Build dependency graph
        let graph = DependencyGraph::new(accesses);

        // Debug: print schedule
        graph.print_schedule();

        Self {
            systems,
            dependency_graph: graph,
        }
    }

    /// Execute all systems in parallel with proper ordering
    pub fn execute_parallel(&mut self, world: &mut World) -> Result<()> {
        // Clone stages to avoid borrowing self while executing
        let stages = self.dependency_graph.stages().to_vec();

        for stage in &stages {
            // Execute all systems in this stage in parallel
            self.execute_stage(stage, world)?;
        }

        Ok(())
    }

    /// Execute a single stage (all systems in parallel)
    fn execute_stage(&mut self, stage: &ExecutionStage, world: &mut World) -> Result<()> {
        // We need to bypass the borrow checker here because we know that
        // the systems in a stage do not conflict with each other.
        // We use UnsafeCell to allow mutable access to systems from multiple threads.

        // Cast to usize to allow passing to threads (usize is Send + Sync)
        let systems_ptr = self.systems.as_mut_ptr() as usize;
        let world_ptr = world as *mut World as usize;

        // Execute all systems in this stage in parallel
        let results: Vec<Result<()>> = stage
            .system_indices
            .par_iter()
            .map(move |&sys_idx| {
                // Safety: sys_idx is guaranteed to be within bounds and unique per thread
                // (or at least distinct from other threads in this stage)
                if sys_idx == usize::MAX {
                    // Dummy check to keep compiler happy if needed, but logic ensures safety
                    return Err(crate::error::EcsError::SystemNotFound);
                }

                let system = unsafe { &mut *(systems_ptr as *mut Box<dyn System>).add(sys_idx) };
                let world = unsafe { &mut *(world_ptr as *mut World) };
                system.run(world)
            })
            .collect();

        // Check for errors
        for result in results {
            result?;
        }

        Ok(())
    }

    /// Get dependency graph for inspection
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::SystemAccess;
    use crate::world::World;

    struct DummySystem {
        name: &'static str,
        access: SystemAccess,
    }

    impl System for DummySystem {
        fn name(&self) -> &'static str {
            self.name
        }

        fn access(&self) -> SystemAccess {
            self.access.clone()
        }

        fn run(&mut self, _world: &mut World) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_parallel_executor_creation() {
        let systems: Vec<Box<dyn System>> = vec![Box::new(DummySystem {
            name: "sys1",
            access: SystemAccess::empty(),
        })];

        let executor = ParallelExecutor::new(systems);
        assert_eq!(executor.systems.len(), 1);
    }
}
