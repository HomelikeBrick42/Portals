use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };

    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    pub const FORWARD: Self = Self::X;
    pub const UP: Self = Self::Y;
    pub const RIGHT: Self = Self::Z;

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn sqr_magnitude(self) -> f32 {
        self.dot(self)
    }

    #[inline]
    pub fn magnitude(self) -> f32 {
        self.sqr_magnitude().sqrt()
    }

    #[inline]
    pub fn normalised(self) -> Self {
        let magnitude = self.magnitude();
        if magnitude > 0.0001 {
            self * magnitude.recip()
        } else {
            Self::ZERO
        }
    }
}

impl AsRef<[f32; 3]> for Vector3 {
    #[inline]
    fn as_ref(&self) -> &[f32; 3] {
        bytemuck::cast_ref(self)
    }
}

impl AsMut<[f32; 3]> for Vector3 {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; 3] {
        bytemuck::cast_mut(self)
    }
}

impl From<[f32; 3]> for Vector3 {
    #[inline]
    fn from([x, y, z]: [f32; 3]) -> Self {
        Self { x, y, z }
    }
}

impl From<Vector3> for [f32; 3] {
    #[inline]
    fn from(Vector3 { x, y, z }: Vector3) -> [f32; 3] {
        [x, y, z]
    }
}

encase::impl_vector!(3, Vector3, f32; using AsRef AsMut From);

impl Add<Vector3> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn add(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<f32> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

impl AddAssign<Vector3> for Vector3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector3) {
        *self = *self + rhs;
    }
}

impl AddAssign<f32> for Vector3 {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        *self = *self + rhs;
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn sub(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<f32> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl SubAssign<Vector3> for Vector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector3) {
        *self = *self - rhs;
    }
}

impl SubAssign<f32> for Vector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        *self = *self - rhs;
    }
}

impl Mul<Vector3> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn mul(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl MulAssign<Vector3> for Vector3 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vector3) {
        *self = *self * rhs;
    }
}

impl MulAssign<f32> for Vector3 {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs;
    }
}

impl Div<Vector3> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn div(self, rhs: Vector3) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;

    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl DivAssign<Vector3> for Vector3 {
    #[inline]
    fn div_assign(&mut self, rhs: Vector3) {
        *self = *self / rhs;
    }
}

impl DivAssign<f32> for Vector3 {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs;
    }
}
