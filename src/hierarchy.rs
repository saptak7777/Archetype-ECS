use crate::entity::EntityId;

/// Parent relationship component
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parent(pub EntityId);

impl Parent {
    pub fn new(parent_id: EntityId) -> Self {
        Self(parent_id)
    }

    pub fn entity_id(&self) -> EntityId {
        self.0
    }
}

/// Children relationship component
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Children {
    children: Vec<EntityId>,
}

impl Children {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: EntityId) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    pub fn remove_child(&mut self, child: EntityId) -> bool {
        if let Some(pos) = self.children.iter().position(|&c| c == child) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, child: EntityId) -> bool {
        self.children.contains(&child)
    }

    pub fn iter(&self) -> impl Iterator<Item = &EntityId> {
        self.children.iter()
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn clear(&mut self) {
        self.children.clear();
    }

    pub fn get_children(&self) -> Vec<EntityId> {
        self.children.clone()
    }
}

impl Default for Children {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks if transform changed (for dirty propagation)
#[derive(Clone, Copy, Debug)]
pub struct TransformChanged {
    pub changed: bool,
}

impl TransformChanged {
    pub fn new(changed: bool) -> Self {
        Self { changed }
    }

    pub fn mark_changed(&mut self) {
        self.changed = true;
    }

    pub fn clear(&mut self) {
        self.changed = false;
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }
}

impl Default for TransformChanged {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn test_children_management() {
        let mut world = World::new();
        let id1 = world
            .spawn((crate::transform::LocalTransform::identity(),))
            .unwrap();
        let id2 = world
            .spawn((crate::transform::LocalTransform::identity(),))
            .unwrap();

        let mut children = Children::new();
        children.add_child(id1);
        assert!(children.contains(id1));
        assert_eq!(children.len(), 1);

        children.add_child(id2);
        assert_eq!(children.len(), 2);

        children.remove_child(id1);
        assert!(!children.contains(id1));
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_children_no_duplicates() {
        let mut world = World::new();
        let id = world
            .spawn((crate::transform::LocalTransform::identity(),))
            .unwrap();

        let mut children = Children::new();
        children.add_child(id);
        children.add_child(id); // Add same child twice
        assert_eq!(children.len(), 1); // Should still be 1
    }

    #[test]
    fn test_transform_changed() {
        let mut changed = TransformChanged::new(false);
        assert!(!changed.is_changed());

        changed.mark_changed();
        assert!(changed.is_changed());

        changed.clear();
        assert!(!changed.is_changed());
    }
}
