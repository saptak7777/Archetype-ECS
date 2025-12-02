//! Convenient re-exports of commonly used types.
//!
//! The prelude can be imported with:
//! ```
//! use aaa_ecs::prelude::*;
//! ```

pub use crate::app::App;
pub use crate::builtin::{
    Children, GlobalTransform, Input, KeyCode, KeyboardInput, MouseButton, MouseInput,
    MousePosition, Parent, Quat, Transform, Vec3,
};
pub use crate::component::Component;
pub use crate::debug::{Diagnostics, WorldInspector};
pub use crate::entity::EntityId;
pub use crate::plugin::Plugin;
pub use crate::query::Query;
pub use crate::reflection::{Reflect, TypeRegistry};
pub use crate::time::{FixedTime, Time};
pub use crate::world::World;
