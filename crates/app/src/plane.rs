use math::{Rotor, Transform, Vector3};
use ray_tracing::{Color, GpuPlane, GpuPortalConnection};
use serde::{Deserialize, Serialize};

use crate::{Hit, Ray};

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
    pub fn transform(&self) -> Transform {
        Transform::translation(self.position).then(Transform::from_rotor(
            Rotor::rotation_xy(self.xy_rotation)
                .then(Rotor::rotation_yz(self.yz_rotation))
                .then(Rotor::rotation_xz(self.xz_rotation)),
        ))
    }

    pub fn intersect(&self, ray: Ray) -> Option<Hit> {
        let transform = self.transform();
        let inverse_transform = transform.reverse();
        let origin = inverse_transform.transform_point(ray.origin);
        let direction = inverse_transform.rotor_part().rotate(ray.direction);

        if origin.y.signum() == direction.y.signum() || direction.y.abs() < 0.001 {
            return None;
        }

        let distance = (origin.y / direction.y).abs();
        let position = ray.origin + ray.direction * distance;
        let normal = transform
            .transform_point(Vector3 {
                x: 0.0,
                y: -direction.y,
                z: 0.0,
            })
            .normalised();
        let front = direction.y < 0.0;

        let local_pos = origin + direction * distance;
        if local_pos.x < self.width * -0.5
            || local_pos.z < self.height * -0.5
            || local_pos.x > self.width * 0.5
            || local_pos.z > self.height * 0.5
        {
            return None;
        }

        Some(Hit {
            distance,
            position,
            normal,
            front,
        })
    }

    pub fn to_gpu(&self) -> GpuPlane {
        let Self {
            name: _,
            position: _,
            xy_rotation: _,
            yz_rotation: _,
            xz_rotation: _,
            color,
            width,
            height,
            checker_count_x,
            checker_count_z,
            checker_darkness,
            ref front_portal,
            ref back_portal,
        } = *self;
        GpuPlane {
            transform: self.transform(),
            color,
            width,
            height,
            checker_count_x,
            checker_count_z,
            checker_darkness,
            front_portal: GpuPortalConnection {
                other_index: front_portal
                    .other_index
                    .map(|index| index as u32)
                    .unwrap_or(u32::MAX),
                flip: front_portal.flip as u32,
            },
            back_portal: GpuPortalConnection {
                other_index: back_portal
                    .other_index
                    .map(|index| index as u32)
                    .unwrap_or(u32::MAX),
                flip: back_portal.flip as u32,
            },
        }
    }
}
