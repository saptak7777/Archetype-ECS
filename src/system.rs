//! System trait and access metadata

use crate::error::Result;
use crate::World;
use std::any::TypeId;

/// System ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemId(pub u32); // Made public

/// System access metadata
#[derive(Debug, Clone)]
pub struct SystemAccess {
    pub reads: Vec<TypeId>,
    pub writes: Vec<TypeId>,
}

impl SystemAccess {
    /// Create empty access
    pub fn empty() -> Self {
        Self {
            reads: Vec::new(),
            writes: Vec::new(),
        }
    }

    /// Check if conflicts with another access
    pub fn conflicts_with(&self, other: &SystemAccess) -> bool {
        // Write-write conflicts
        for w1 in &self.writes {
            for w2 in &other.writes {
                if w1 == w2 {
                    return true;
                }
            }
        }

        // Write-read conflicts
        for w in &self.writes {
            for r in &other.reads {
                if w == r {
                    return true;
                }
            }
        }

        // Read-write conflicts
        for r in &self.reads {
            for w in &other.writes {
                if r == w {
                    return true;
                }
            }
        }

        false
    }
}

/// System trait
pub trait System: Send + Sync {
    /// Get system access metadata
    fn access(&self) -> SystemAccess;

    /// Get system name
    fn name(&self) -> &'static str;

    /// Run system
    fn run(&mut self, world: &World) -> Result<()>;
}

/// Boxed system
pub type BoxedSystem = Box<dyn System>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_access_conflicts() {
        let mut access1 = SystemAccess::empty();
        access1.writes.push(TypeId::of::<i32>());

        let mut access2 = SystemAccess::empty();
        access2.writes.push(TypeId::of::<i32>());

        assert!(access1.conflicts_with(&access2));
    }

    #[test]
    fn test_system_access_no_conflicts() {
        let mut access1 = SystemAccess::empty();
        access1.reads.push(TypeId::of::<i32>());

        let mut access2 = SystemAccess::empty();
        access2.reads.push(TypeId::of::<i32>());

        assert!(!access1.conflicts_with(&access2));
    }
}
