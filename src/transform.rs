use serde::{Deserialize, Serialize};

// Re-export glam types for standardization and ease of use
pub use glam::{Mat4, Quat, Vec3};

/// Local transform (relative to parent)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocalTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl LocalTransform {
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn with_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn with_rotation(rotation: Quat) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// Legacy constructor for compatibility if needed
    pub fn from_translation(position: Vec3) -> Self {
        Self::with_position(position)
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Global transform (world space)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl GlobalTransform {
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Combine parent global + local child → global child
    pub fn from_local(parent: &GlobalTransform, child: &LocalTransform) -> Self {
        // Correct order: Scale -> Rotate -> Translate
        // Global = ParentGlobal * Local
        let rotated_pos = parent.rotation * (child.position * parent.scale);
        let new_position = parent.position + rotated_pos;
        let new_rotation = parent.rotation * child.rotation;
        let new_scale = parent.scale * child.scale;

        GlobalTransform {
            position: new_position,
            rotation: new_rotation,
            scale: new_scale,
        }
    }

    /// Convert global to local (inverse operation)
    pub fn to_local(&self, parent: &GlobalTransform) -> LocalTransform {
        let rel_pos = self.position - parent.position;
        let inv_rot = parent.rotation.inverse();

        // Reciprocal handle potential zero scale safely
        let inv_scale = Vec3::new(
            if parent.scale.x != 0.0 {
                1.0 / parent.scale.x
            } else {
                0.0
            },
            if parent.scale.y != 0.0 {
                1.0 / parent.scale.y
            } else {
                0.0
            },
            if parent.scale.z != 0.0 {
                1.0 / parent.scale.z
            } else {
                0.0
            },
        );

        let position = (inv_rot * rel_pos) * inv_scale;
        let rotation = inv_rot * self.rotation;
        let scale = self.scale * inv_scale;

        LocalTransform {
            position,
            rotation,
            scale,
        }
    }

    /// Returns the position vector
    pub fn translation(&self) -> Vec3 {
        self.position
    }
}

impl Default for GlobalTransform {
    fn default() -> Self {
        Self::identity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(1.0, 0.0, 0.0);

        let sum = v1 + v2;
        assert_eq!(sum.x, 2.0);
        assert_eq!(sum.y, 2.0);

        let scaled = v1 * 2.0;
        assert_eq!(scaled.x, 2.0);
        assert_eq!(scaled.y, 4.0);
    }

    #[test]
    fn test_vec3_length() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert!((v.length() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_quat_identity() {
        let q = Quat::IDENTITY;
        let v = Vec3::X;
        let rotated = q * v;

        assert!((rotated.x - 1.0).abs() < 0.001);
        assert!((rotated.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_global_from_local() {
        let parent = GlobalTransform {
            position: Vec3::new(10.0, 20.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let child = LocalTransform {
            position: Vec3::new(5.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let global = GlobalTransform::from_local(&parent, &child);
        assert!((global.position.x - 15.0).abs() < 0.001);
        assert!((global.position.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_global_from_local_with_scale() {
        let parent = GlobalTransform {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(2.0),
        };

        let child = LocalTransform {
            position: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let global = GlobalTransform::from_local(&parent, &child);
        assert!((global.position.x - 2.0).abs() < 0.001);
    }
}
