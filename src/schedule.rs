//! Schedule builder with dependency graph
//!
//! Constructs system execution schedule via topological sort.

use std::collections::{HashMap, VecDeque};

use crate::error::{EcsError, Result};
use crate::system::{BoxedSystem, System, SystemAccess, SystemId};

/// System node in dependency graph
#[derive(Debug, Clone)]
pub struct SystemNode {
    pub id: SystemId,
    pub access: SystemAccess,
}

/// Dependency graph for systems
pub struct SystemGraph {
    pub nodes: Vec<SystemNode>,
    pub edges: HashMap<SystemId, Vec<SystemId>>,
    pub reverse_edges: HashMap<SystemId, Vec<SystemId>>,
}

impl SystemGraph {
    /// Build graph from systems
    pub fn build(systems: &[BoxedSystem]) -> Self {
        let mut nodes = Vec::new();
        let mut edges: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
        let mut reverse_edges: HashMap<SystemId, Vec<SystemId>> = HashMap::new();

        // Create nodes
        for (i, system) in systems.iter().enumerate() {
            let id = SystemId(i as u32);
            let access = system.access();
            nodes.push(SystemNode { id, access });
            edges.insert(id, Vec::new());
            reverse_edges.insert(id, Vec::new());
        }

        // Build edges (conflicts)
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let id_a = nodes[i].id;
                let id_b = nodes[j].id;

                if nodes[i].access.conflicts_with(&nodes[j].access) {
                    edges.get_mut(&id_a).unwrap().push(id_b);
                    reverse_edges.get_mut(&id_b).unwrap().push(id_a);
                }
            }
        }

        Self {
            nodes,
            edges,
            reverse_edges,
        }
    }

    /// Topological sort (Kahn's algorithm)
    pub fn topological_sort(&self) -> Result<Vec<SystemId>> {
        let mut in_degree: HashMap<SystemId, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees
        for node in &self.nodes {
            in_degree.insert(
                node.id,
                self.reverse_edges.get(&node.id).map_or(0, |v| v.len()),
            );
        }

        // Find nodes with 0 in-degree
        for node in &self.nodes {
            if in_degree[&node.id] == 0 {
                queue.push_back(node.id);
            }
        }

        // Process queue
        while let Some(id) = queue.pop_front() {
            result.push(id);

            if let Some(neighbors) = self.edges.get(&id) {
                for &neighbor in neighbors {
                    let degree = in_degree.get_mut(&neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != self.nodes.len() {
            return Err(EcsError::SystemCycleDetected);
        }

        Ok(result)
    }
}

/// Stage of systems that can run in parallel
#[derive(Debug, Clone)]
pub struct Stage {
    pub systems: Vec<SystemId>,
}

impl Stage {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Try to add system to this stage
    pub fn try_add(
        &mut self,
        system_id: SystemId,
        access: &SystemAccess,
        _graph: &SystemGraph,
    ) -> bool {
        // Check conflicts with existing systems
        for &existing_id in &self.systems {
            let existing_node = _graph.nodes.iter().find(|n| n.id == existing_id).unwrap();

            if access.conflicts_with(&existing_node.access) {
                return false;
            }
        }

        self.systems.push(system_id);
        true
    }
}

impl Default for Stage {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete execution schedule
pub struct Schedule {
    pub(crate) systems: Vec<BoxedSystem>,
    pub(crate) stages: Vec<Stage>,
    pub(crate) graph: Option<SystemGraph>,
}

impl Default for Schedule {
    fn default() -> Self {
        Self::new()
    }
}

impl Schedule {
    /// Build a schedule directly from a vector of systems
    pub fn from_systems(systems: Vec<BoxedSystem>) -> Result<Self> {
        Self {
            systems,
            stages: Vec::new(),
            graph: None,
        }
        .build()
    }

    /// Create an empty schedule
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            stages: Vec::new(),
            graph: None,
        }
    }

    /// Convenience constructor for chaining
    pub fn with_system(mut self, system: BoxedSystem) -> Self {
        self.add_system(system);
        self
    }

    /// Add a system to the schedule definition
    pub fn add_system(&mut self, system: BoxedSystem) {
        self.systems.push(system);
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.graph = None;
        self.stages.clear();
    }

    /// Get mutable reference to a system by name
    pub fn get_system_mut(&mut self, name: &str) -> Option<&mut (dyn System + 'static)> {
        self.systems
            .iter_mut()
            .find(|sys| sys.name() == name)
            .map(|sys| sys.as_mut())
    }

    /// Finalize schedule (topological sort + stage grouping)
    pub fn build(mut self) -> Result<Self> {
        self.rebuild()?;
        Ok(self)
    }

    /// Ensure schedule is built (used internally by executor)
    pub(crate) fn ensure_built(&mut self) -> Result<()> {
        if self.graph.is_none() {
            self.rebuild()?;
        }
        Ok(())
    }

    fn rebuild(&mut self) -> Result<()> {
        let graph = SystemGraph::build(&self.systems);
        let sorted = graph.topological_sort()?;

        // Group into stages (greedy)
        let mut stages = Vec::new();
        let mut current_stage = Stage::new();

        for &system_id in &sorted {
            let node = graph.nodes.iter().find(|n| n.id == system_id).unwrap();

            if !current_stage.try_add(system_id, &node.access, &graph) {
                if !current_stage.systems.is_empty() {
                    stages.push(current_stage);
                    current_stage = Stage::new();
                }
                current_stage.systems.push(system_id);
            }
        }

        if !current_stage.systems.is_empty() {
            stages.push(current_stage);
        }

        self.graph = Some(graph);
        self.stages = stages;
        Ok(())
    }

    /// Get stage count
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Get systems in stage
    pub fn stage_system_count(&self, stage_idx: usize) -> usize {
        self.stages.get(stage_idx).map_or(0, |s| s.systems.len())
    }

    /// Total number of registered systems
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    pub(crate) fn system_mut_by_id(&mut self, id: SystemId) -> Option<&mut BoxedSystem> {
        self.systems.get_mut(id.0 as usize)
    }

    pub(crate) fn stage_plan(&self) -> Vec<Vec<SystemId>> {
        self.stages
            .iter()
            .map(|stage| stage.systems.clone())
            .collect()
    }

    /// Get system accesses for dependency analysis
    pub fn get_accesses(&self) -> Vec<SystemAccess> {
        self.systems.iter().map(|s| s.access()).collect()
    }

    /// Build parallel execution stages
    pub fn analyze_parallelization(&self) -> crate::dependency::DependencyGraph {
        use crate::dependency::DependencyGraph;
        DependencyGraph::new(self.get_accesses())
    }

    /// Print execution plan
    pub fn print_execution_plan(&self) {
        let graph = self.analyze_parallelization();
        graph.print_schedule();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_creation() {
        let stage = Stage::new();
        assert_eq!(stage.systems.len(), 0);
    }
}
