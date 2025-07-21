use bytemuck::{Pod, Zeroable};
use encase::ShaderType;

#[derive(Debug, Clone, Copy, Zeroable, Pod, ShaderType)]
#[repr(C)]
pub struct Rotor {
    pub s: f32,
    pub e12: f32,
    pub e13: f32,
    pub e23: f32,
}

impl Rotor {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
    };
}
