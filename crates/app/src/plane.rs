use math::{Rotor, Transform, Vector3};
use ray_tracing::{Color, GpuPlane};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Plane {
    pub name: String,
    pub position: Vector3,
    pub xy_rotation: f32,
    pub yz_rotation: f32,
    pub xz_rotation: f32,
    pub color: Color,
    pub width: f32,
    pub height: f32,
    pub checker_count_x: u32,
    pub checker_count_z: u32,
    pub checker_darkness: f32,
    pub front_portal: PortalConnection,
    pub back_portal: PortalConnection,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PortalConnection {
    pub other_index: Option<usize>,
    pub flip: bool,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            name: "Default Plane".into(),
            position: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            xy_rotation: 0.0,
            yz_rotation: 0.0,
            xz_rotation: 0.0,
            color: Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            width: 1.0,
            height: 1.0,
            checker_count_x: 1,
            checker_count_z: 1,
            checker_darkness: 0.5,
            front_portal: PortalConnection::default(),
            back_portal: PortalConnection::default(),
        }
    }
}

impl Plane {
    pub fn to_gpu(&self) -> GpuPlane {
        let Self {
            name: _,
            position,
            xy_rotation,
            yz_rotation,
            xz_rotation,
            color,
            width,
            height,
            checker_count_x,
            checker_count_z,
            checker_darkness,
            front_portal: _,
            back_portal: _,
        } = *self;
        GpuPlane {
            transform: Transform::translation(position).then(Transform::from_rotor(
                Rotor::rotation_xy(xy_rotation)
                    .then(Rotor::rotation_yz(yz_rotation))
                    .then(Rotor::rotation_xz(xz_rotation)),
            )),
            color,
            width,
            height,
            checker_count_x,
            checker_count_z,
            checker_darkness,
        }
    }
}
