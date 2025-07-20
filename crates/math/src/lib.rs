use encase::ShaderType;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Transform {
    pub s: f32,
    pub e01: f32,
    pub e02: f32,
    pub e03: f32,
    pub e12: f32,
    pub e13: f32,
    pub e23: f32,
    pub e0123: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e0123: 0.0,
    };
}
