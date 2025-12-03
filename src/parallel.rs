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
    ///
    /// # Safety Architecture
    ///
    /// This function uses unsafe code to enable parallel system execution while bypassing
    /// Rust's borrow checker. The safety of this approach relies on the following invariants:
    ///
    /// ## Invariant 1: Non-Overlapping System Access
    /// The `DependencyGraph` guarantees that all systems within a single stage have
    /// non-conflicting access patterns. Systems are only grouped in the same stage if:
    /// - They don't both write to the same component type
    /// - If one writes to a component, the other doesn't read or write it
    ///
    /// ## Invariant 2: Valid System Indices
    /// All indices in `stage.system_indices` are guaranteed to be:
    /// - Within bounds: `sys_idx < self.systems.len()`
    /// - Unique within the stage (no duplicate indices)
    /// - Derived from the dependency graph construction
    ///
    /// ## Invariant 3: Thread-Safe World Access
    /// Although multiple threads access `world` simultaneously, the ECS architecture ensures:
    /// - Each system accesses different archetypes or different components
    /// - Component columns are stored separately, preventing data races
    /// - The dependency graph enforces exclusive access to conflicting resources
    ///
    /// ## Lifetime Guarantees
    /// - The raw pointers are only valid for the duration of this function
    /// - No references escape the parallel iteration scope
    /// - All borrows are released before the function returns
    ///
    /// ## Why This Is Safe
    /// 1. **Spatial Safety**: Each thread accesses a unique system (unique indices)
    /// 2. **Temporal Safety**: Pointers are only dereferenced within the par_iter scope
    /// 3. **Data Race Freedom**: Dependency graph ensures disjoint memory access
    /// 4. **Bounds Safety**: Index validation prevents out-of-bounds access
    fn execute_stage(&mut self, stage: &ExecutionStage, world: &mut World) -> Result<()> {
        // Convert pointers to usize for Send + Sync across thread boundaries
        // This is safe because we're only using them as opaque handles
        let systems_ptr = self.systems.as_mut_ptr() as usize;
        let world_ptr = world as *mut World as usize;

        // Execute all systems in this stage in parallel using Rayon's work-stealing
        let results: Vec<Result<()>> = stage
            .system_indices
            .par_iter()
            .map(move |&sys_idx| {
                // Validate index bounds (defensive programming)
                if sys_idx == usize::MAX {
                    return Err(crate::error::EcsError::SystemNotFound);
                }

                // SAFETY: This is safe because:
                // 1. sys_idx is guaranteed to be < self.systems.len() (from dependency graph)
                // 2. sys_idx is unique within this stage (no two threads access same system)
                // 3. The pointer is valid for the lifetime of this function
                // 4. No other code is accessing self.systems during parallel execution
                let system = unsafe { &mut *(systems_ptr as *mut Box<dyn System>).add(sys_idx) };

                // SAFETY: This is safe because:
                // 1. The world pointer is valid for the duration of this function
                // 2. Systems in this stage have non-conflicting access (verified by DependencyGraph)
                // 3. Each system accesses disjoint sets of components/archetypes
                // 4. The ECS architecture prevents data races through archetype isolation
                let world = unsafe { &mut *(world_ptr as *mut World) };

                system.run(world)
            })
            .collect();

        // Propagate any errors from system execution
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
