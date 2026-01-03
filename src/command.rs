// Copyright 2024 Saptak Santra
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Command buffer with struct variants

use crate::component::Component;
use crate::entity::EntityId;
use crate::error::Result;
pub use crate::world::World;

/// Type alias for world mutation closures
pub type CommandClosure = Box<dyn FnOnce(&mut World) -> Result<()> + Send>;

/// Deferred command for world mutations  
/// Deferred command for world mutations  
pub enum Command {
    /// Spawn entity with closure
    Spawn(CommandClosure),

    /// Despawn entity
    Despawn(EntityId),

    /// Custom world mutation
    Custom(CommandClosure),
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Spawn(_) => write!(f, "Spawn(...)"),
            Command::Despawn(e) => f.debug_tuple("Despawn").field(e).finish(),
            Command::Custom(_) => write!(f, "Custom(...)"),
        }
    }
}

/// Command buffer for deferred operations
#[derive(Default)]
pub struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBuffer {
    /// Create new command buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            commands: Vec::with_capacity(capacity),
        }
    }
}

impl CommandBuffer {
    /// Queue spawn command with a stateful closure
    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World) -> Result<()> + Send + 'static,
    {
        self.commands.push(Command::Spawn(Box::new(f)));
    }

    /// Queue despawn command
    pub fn despawn(&mut self, entity: EntityId) {
        self.commands.push(Command::Despawn(entity));
    }

    /// Queue a custom world mutation
    pub fn add<F>(&mut self, f: F)
    where
        F: FnOnce(&mut World) -> Result<()> + Send + 'static,
    {
        self.commands.push(Command::Custom(Box::new(f)));
    }

    /// Queue add component command
    pub fn add_component<T: Component>(&mut self, entity: EntityId, component: T) {
        self.add(move |world| world.add_component(entity, component));
    }

    /// Queue remove component command
    pub fn remove_component<T: Component>(&mut self, entity: EntityId) {
        self.add(move |world| world.remove_component::<T>(entity).map(|_| ()));
    }

    /// Apply all commands to the world and clear the buffer
    pub fn apply(&mut self, world: &mut World) -> Result<()> {
        for command in self.commands.drain(..) {
            match command {
                Command::Spawn(f) => {
                    f(world)?;
                }
                Command::Despawn(entity) => {
                    world.despawn(entity)?;
                }
                Command::Custom(f) => {
                    f(world)?;
                }
            }
        }
        Ok(())
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::Key;

    #[test]
    fn test_command_buffer() {
        let mut buffer = CommandBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        let entity = EntityId::null();
        buffer.despawn(entity);

        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_command_buffer_clear() {
        let mut buffer = CommandBuffer::new();
        let entity = EntityId::null();
        buffer.despawn(entity);
        buffer.clear();
        assert_eq!(buffer.len(), 0);
    }
}
