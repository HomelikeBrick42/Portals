use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
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
