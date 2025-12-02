use std::ops::{Add, Mul, Sub};

/// 3D Vector (for 2D games, just use z=0)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn one() -> Self {
        Self {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }

    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalized(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

/// Quaternion for rotations
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn identity() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();

        Self {
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
            w: cr * cp * cy + sr * sp * sy,
        }
    }

    pub fn inverse(&self) -> Quat {
        Quat {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn multiply(&self, other: Quat) -> Quat {
        Quat {
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
        }
    }

    /// Rotate a vector by this quaternion
    pub fn rotate_vector(&self, v: Vec3) -> Vec3 {
        let q_inv = self.inverse();
        let v_quat = Quat {
            x: v.x,
            y: v.y,
            z: v.z,
            w: 0.0,
        };

        let result = self.multiply(v_quat).multiply(q_inv);
        Vec3 {
            x: result.x,
            y: result.y,
            z: result.z,
        }
    }
}

/// Local transform (relative to parent)
#[derive(Clone, Copy, Debug)]
pub struct LocalTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl LocalTransform {
    pub fn identity() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quat::identity(),
            scale: Vec3::one(),
        }
    }

    pub fn with_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::identity(),
            scale: Vec3::one(),
        }
    }

    pub fn with_rotation(rotation: Quat) -> Self {
        Self {
            position: Vec3::zero(),
            rotation,
            scale: Vec3::one(),
        }
    }

    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quat::identity(),
            scale,
        }
    }
}

impl Default for LocalTransform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Global transform (world space)
#[derive(Clone, Copy, Debug)]
pub struct GlobalTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl GlobalTransform {
    pub fn identity() -> Self {
        Self {
            position: Vec3::zero(),
            rotation: Quat::identity(),
            scale: Vec3::one(),
        }
    }

    /// Combine parent global + local child → global child
    pub fn from_local(parent: &GlobalTransform, child: &LocalTransform) -> Self {
        let scaled_pos = child.position * parent.scale;
        let rotated_pos = parent.rotation.rotate_vector(scaled_pos);
        let new_position = parent.position + rotated_pos;
        let new_rotation = parent.rotation.multiply(child.rotation);
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
        let position = inv_rot.rotate_vector(rel_pos)
            * Vec3::new(
                1.0 / parent.scale.x,
                1.0 / parent.scale.y,
                1.0 / parent.scale.z,
            );

        let rotation = inv_rot.multiply(self.rotation);

        let scale = Vec3 {
            x: self.scale.x / parent.scale.x,
            y: self.scale.y / parent.scale.y,
            z: self.scale.z / parent.scale.z,
        };

        LocalTransform {
            position,
            rotation,
            scale,
        }
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
        let q = Quat::identity();
        let v = Vec3::new(1.0, 0.0, 0.0);
        let rotated = q.rotate_vector(v);

        assert!((rotated.x - 1.0).abs() < 0.001);
        assert!((rotated.y - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_global_from_local() {
        let parent = GlobalTransform {
            position: Vec3::new(10.0, 20.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::one(),
        };

        let child = LocalTransform {
            position: Vec3::new(5.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::one(),
        };

        let global = GlobalTransform::from_local(&parent, &child);
        assert!((global.position.x - 15.0).abs() < 0.001);
        assert!((global.position.y - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_global_from_local_with_scale() {
        let parent = GlobalTransform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::new(2.0, 2.0, 2.0),
        };

        let child = LocalTransform {
            position: Vec3::new(1.0, 0.0, 0.0),
            rotation: Quat::identity(),
            scale: Vec3::one(),
        };

        let global = GlobalTransform::from_local(&parent, &child);
        assert!((global.position.x - 2.0).abs() < 0.001);
    }
}
