use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

#[derive(Debug, Clone, Copy, Zeroable, Pod, Serialize, Deserialize)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl AsRef<[f32; 3]> for Color {
    fn as_ref(&self) -> &[f32; 3] {
        bytemuck::cast_ref(self)
    }
}

impl AsMut<[f32; 3]> for Color {
    fn as_mut(&mut self) -> &mut [f32; 3] {
        bytemuck::cast_mut(self)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self { r, g, b }
    }
}

impl From<Color> for [f32; 3] {
    fn from(Color { r, g, b }: Color) -> [f32; 3] {
        [r, g, b]
    }
}

encase::impl_vector!(3, Color, f32; using AsRef AsMut From);
