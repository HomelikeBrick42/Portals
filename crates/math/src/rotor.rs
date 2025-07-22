use bytemuck::{Pod, Zeroable};
use encase::ShaderType;

use crate::Vector3;

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

    #[inline]
    #[must_use]
    pub fn rotation_xy(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e12: sin,
            ..Self::IDENTITY
        }
    }

    #[inline]
    #[must_use]
    pub fn rotation_xz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e13: sin,
            ..Self::IDENTITY
        }
    }

    #[inline]
    #[must_use]
    pub fn rotation_yz(angle: f32) -> Self {
        let (sin, cos) = (angle * 0.5).sin_cos();
        Self {
            s: cos,
            e23: sin,
            ..Self::IDENTITY
        }
    }

    #[inline]
    #[must_use]
    pub const fn then(self, then: Self) -> Self {
        then.after(self)
    }

    #[inline]
    #[must_use]
    pub const fn after(self, after: Self) -> Self {
        /*
            (a1 + b1*e1*e2 + c1*e1*e3 + d1*e2*e3)
            *
            (a2 + b2*e1*e2 + c2*e1*e3 + d2*e2*e3)

            =

            (a1*a2 + -1*b1*b2 + -1*c1*c2 + -1*d1*d2)
            e1*e2*(a1*b2 + a2*b1 + c2*d1 + -1*c1*d2)
            e1*e3*(a1*c2 + a2*c1 + b1*d2 + -1*b2*d1)
            e2*e3*(a1*d2 + a2*d1 + b2*c1 + -1*b1*c2)
        */

        let Self {
            s: a1,
            e12: b1,
            e13: c1,
            e23: d1,
        } = self;
        let Self {
            s: a2,
            e12: b2,
            e13: c2,
            e23: d2,
        } = after;
        Self {
            s: a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            e12: a1 * b2 + a2 * b1 + c2 * d1 - c1 * d2,
            e13: a1 * c2 + a2 * c1 + b1 * d2 - b2 * d1,
            e23: a1 * d2 + a2 * d1 + b2 * c1 - b1 * c2,
        }
    }

    #[inline]
    #[must_use]
    pub const fn rotate(self, point: Vector3) -> Vector3 {
        /*
            (a + -1*b*e1*e2 + -1*c*e1*e3 + -1*d*e2*e3)
            *
            (0*e1*e2*e3 + x*e0*e3*e2 + y*e0*e1*e3 + z*e0*e2*e1)
            *
            (a + b*e1*e2 + c*e1*e3 + d*e2*e3)

            =

            e0*e1*e2*(c*c*z + d*d*z + -2*a*c*x + -2*a*d*y + -2*b*d*x + -1*a*a*z + -1*b*b*z + 2*b*c*y)
            e0*e1*e3*(a*a*y + c*c*y + -2*a*d*z + -2*b*c*z + -2*c*d*x + -1*b*b*y + -1*d*d*y + 2*a*b*x)
            e0*e2*e3*(b*b*x + c*c*x + -2*b*d*z + -1*a*a*x + -1*d*d*x + 2*a*b*y + 2*a*c*z + 2*c*d*y)
        */

        let Self {
            s: a,
            e12: b,
            e13: c,
            e23: d,
        } = self;
        let Vector3 { x, y, z } = point;

        let e012 = c * c * z + d * d * z
            - 2.0 * a * c * x
            - 2.0 * a * d * y
            - 2.0 * b * d * x
            - a * a * z
            - b * b * z
            + 2.0 * b * c * y;
        let e013 = a * a * y + c * c * y
            - 2.0 * a * d * z
            - 2.0 * b * c * z
            - 2.0 * c * d * x
            - b * b * y
            - d * d * y
            + 2.0 * a * b * x;
        let e023 = b * b * x + c * c * x - 2.0 * b * d * z - a * a * x - d * d * x
            + 2.0 * a * b * y
            + 2.0 * a * c * z
            + 2.0 * c * d * y;

        Vector3 {
            x: -e023,
            y: e013,
            z: -e012,
        }
    }
}
