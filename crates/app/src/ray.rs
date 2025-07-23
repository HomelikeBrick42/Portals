use math::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}

#[derive(Debug, Clone, Copy)]
pub struct Hit {
    pub distance: f32,
    pub position: Vector3,
    pub normal: Vector3,
    pub front: bool,
}
