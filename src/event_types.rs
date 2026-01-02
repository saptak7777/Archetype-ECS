use crate::entity::EntityId;
use crate::event_bus::Event;
use std::any::{Any, TypeId};

/// Macro for defining events with automatic Event trait implementation
#[macro_export]
macro_rules! define_event {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field:ident : $ty:ty),* $(,)?
        }
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
        }
    };
    
    // Support for unit structs (no fields)
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

/// Player took damage
#[derive(Clone, Debug)]
pub struct PlayerDamaged {
    pub entity: EntityId,
    pub damage: f32,
    pub source: String,
}

impl Event for PlayerDamaged {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "PlayerDamaged"
    }
}

/// Enemy defeated
#[derive(Clone, Debug)]
pub struct EnemyDefeated {
    pub entity: EntityId,
    pub reward: u32,
}

impl Event for EnemyDefeated {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "EnemyDefeated"
    }
}

/// Player level up
#[derive(Clone, Debug)]
pub struct PlayerLevelUp {
    pub entity: EntityId,
    pub new_level: u32,
}

impl Event for PlayerLevelUp {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "PlayerLevelUp"
    }
}

/// Game state changed
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    Menu,
    GameOver,
}

#[derive(Clone, Debug)]
pub struct GameStateChanged {
    pub old_state: GameState,
    pub new_state: GameState,
}

impl Event for GameStateChanged {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "GameStateChanged"
    }
}

/// Input action
#[derive(Clone, Debug)]
pub struct InputAction {
    pub action: String,
    pub value: f32,
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
}

/// Inventory item added
#[derive(Clone, Debug)]
pub struct ItemAdded {
    pub entity: EntityId,
    pub item_id: String,
    pub quantity: u32,
}

impl Event for ItemAdded {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "ItemAdded"
    }
}

/// Collision occurred
#[derive(Clone, Debug)]
pub struct Collision {
    pub entity_a: EntityId,
    pub entity_b: EntityId,
}

impl Event for Collision {
    fn event_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn event_name(&self) -> &str {
        "Collision"
    }
}
