use crate::entity::EntityId;
use crate::event_bus::Event;
use std::any::{Any, TypeId};

/// Macro for defining events with automatic Event trait implementation
#[macro_export]
macro_rules! define_event {
    // Struct with fields and optional validation
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $ty:ty),* $(,)?
        }
        $(validate($this:ident) $validate_body:block)?
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        $vis struct $name {
            $(pub $field : $ty),*
        }

        impl Event for $name {
            fn event_type_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn event_name(&self) -> &str {
                stringify!($name)
            }

            fn validate(&self) -> $crate::error::Result<()> {
                $(
                    let $this = self;
                    return $validate_body;
                )?
                #[allow(unreachable_code)]
                Ok(())
            }
        }
    };

    // Unit struct (no fields)
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
    ) => {
        $(#[$meta])*
        #[derive(Clone, Debug)]
        $vis struct $name;

        impl Event for $name {
            fn event_type_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }

            fn as_any(&self) -> &dyn Any {
                self
            }

            fn event_name(&self) -> &str {
                stringify!($name)
            }
        }
    };
}

// Game State Enum (not an event itself, but data for one)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    Menu,
    GameOver,
}

// ==================================================================================
// Event Definitions
// ==================================================================================

define_event! {
    /// Player took damage
    pub struct PlayerDamaged {
        entity: EntityId,
        damage: f32,
        source: String,
    }
    validate(ev) {
        if ev.damage < 0.0 {
            return Err(crate::error::EcsError::ValidationError(
                "Damage cannot be negative".into(),
            ));
        }
        Ok(())
    }
}

define_event! {
    /// Enemy defeated
    pub struct EnemyDefeated {
        entity: EntityId,
        reward: u32,
    }
}

define_event! {
    /// Player level up
    pub struct PlayerLevelUp {
        entity: EntityId,
        new_level: u32,
    }
}

define_event! {
    /// Game state changed
    pub struct GameStateChanged {
        old_state: GameState,
        new_state: GameState,
    }
}

// Manual definition for InputAction because of complex new() method and specific field types
// The macro doesn't support custom impl blocks easily alongside the definition without more complexity.
/// Input action
#[derive(Clone, Debug)]
pub struct InputAction {
    pub action: smallvec::SmallVec<[u8; 32]>,
    pub value: f32,
}

impl InputAction {
    pub fn new(action: &str, value: f32) -> Self {
        Self {
            action: smallvec::SmallVec::from_slice(action.as_bytes()),
            value,
        }
    }

    pub fn action_name(&self) -> &str {
        std::str::from_utf8(&self.action).unwrap_or("InvalidUTF8")
    }
}

impl Event for InputAction {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "InputAction"
    }

    fn validate(&self) -> crate::error::Result<()> {
        if self.action.is_empty() {
            return Err(crate::error::EcsError::ValidationError(
                "Action name cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

define_event! {
    /// Inventory item added
    pub struct ItemAdded {
        entity: EntityId,
        item_id: String,
        quantity: u32,
    }
}

define_event! {
    /// Collision occurred
    pub struct Collision {
        entity_a: EntityId,
        entity_b: EntityId,
    }
}
