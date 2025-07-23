use bytemuck::{Pod, Zeroable};
use encase::ShaderType;
use serde::{Deserialize, Serialize};

use crate::{Rotor, Vector3};

#[derive(Debug, Clone, Copy, Zeroable, Pod, ShaderType, Serialize, Deserialize)]
#[repr(C)]
pub struct Transform {
    pub s: f32,
    pub e12: f32,
    pub e13: f32,
    pub e23: f32,
    pub e01: f32,
    pub e02: f32,
    pub e03: f32,
    pub e0123: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        s: 1.0,
        e12: 0.0,
        e13: 0.0,
        e23: 0.0,
        e01: 0.0,
        e02: 0.0,
        e03: 0.0,
        e0123: 0.0,
    };

    #[inline]
    #[must_use]
    pub const fn translation(offset: Vector3) -> Self {
        Self {
            e01: offset.x * 0.5,
            e02: offset.y * 0.5,
            e03: offset.z * 0.5,
            ..Self::IDENTITY
        }
    }

    #[inline]
    #[must_use]
    pub fn rotation_xy(angle: f32) -> Self {
        Self::from_rotor(Rotor::rotation_xy(angle))
    }

    #[inline]
    #[must_use]
    pub fn rotation_xz(angle: f32) -> Self {
        Self::from_rotor(Rotor::rotation_xz(angle))
    }

    #[inline]
    #[must_use]
    pub fn rotation_yz(angle: f32) -> Self {
        Self::from_rotor(Rotor::rotation_yz(angle))
    }

    #[inline]
    #[must_use]
    pub const fn from_rotor(rotor: Rotor) -> Self {
        let Rotor { s, e12, e13, e23 } = rotor;
        Self {
            s,
            e12,
            e13,
            e23,
            e01: 0.0,
            e02: 0.0,
            e03: 0.0,
            e0123: 0.0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn rotor_part(self) -> Rotor {
        let Self {
            s,
            e12,
            e13,
            e23,
            e01: _,
            e02: _,
            e03: _,
            e0123: _,
        } = self;
        Rotor { s, e12, e13, e23 }
    }

    #[inline]
    #[must_use]
    pub const fn reverse(self) -> Self {
        let Self {
            s,
            e12,
            e13,
            e23,
            e01,
            e02,
            e03,
            e0123,
        } = self;
        Self {
            s,
            e12: -e12,
            e13: -e13,
            e23: -e23,
            e01: -e01,
            e02: -e02,
            e03: -e03,
            e0123,
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
            (a1 + b1*e1*e2 + c1*e1*e3 + d1*e2*e3 + e1*e0*e1 + f1*e0*e2 + g1*e0*e3 + h1*e0*e1*e2*e3)
            *
            (a2 + b2*e1*e2 + c2*e1*e3 + d2*e2*e3 + e2*e0*e1 + f2*e0*e2 + g2*e0*e3 + h2*e0*e1*e2*e3)

            =

            (a1*a2 + -1*b1*b2 + -1*c1*c2 + -1*d1*d2)
            e1*e2*(a1*b2 + a2*b1 + c2*d1 + -1*c1*d2)
            e1*e3*(a1*c2 + a2*c1 + b1*d2 + -1*b2*d1)
            e2*e3*(a1*d2 + a2*d1 + b2*c1 + -1*b1*c2)
            e0*e1*(a1*e2 + a2*e1 + b1*f2 + c1*g2 + -1*b2*f1 + -1*c2*g1 + -1*d1*h2 + -1*d2*h1)
            e0*e2*(a1*f2 + a2*f1 + b2*e1 + c1*h2 + c2*h1 + d1*g2 + -1*b1*e2 + -1*d2*g1)
            e0*e3*(a1*g2 + a2*g1 + c2*e1 + d2*f1 + -1*b1*h2 + -1*b2*h1 + -1*c1*e2 + -1*d1*f2)
            e0*e1*e2*e3*(a1*h2 + a2*h1 + b1*g2 + b2*g1 + d1*e2 + d2*e1 + -1*c1*f2 + -1*c2*f1)
        */

        let Self {
            s: a1,
            e12: b1,
            e13: c1,
            e23: d1,
            e01: e1,
            e02: f1,
            e03: g1,
            e0123: h1,
        } = self;
        let Self {
            s: a2,
            e12: b2,
            e13: c2,
            e23: d2,
            e01: e2,
            e02: f2,
            e03: g2,
            e0123: h2,
        } = after;
        Self {
            s: a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2,
            e12: a1 * b2 + a2 * b1 + c2 * d1 - c1 * d2,
            e13: a1 * c2 + a2 * c1 + b1 * d2 - b2 * d1,
            e23: a1 * d2 + a2 * d1 + b2 * c1 - b1 * c2,
            e01: a1 * e2 + a2 * e1 + b1 * f2 + c1 * g2 - b2 * f1 - c2 * g1 - d1 * h2 - d2 * h1,
            e02: a1 * f2 + a2 * f1 + b2 * e1 + c1 * h2 + c2 * h1 + d1 * g2 - b1 * e2 - d2 * g1,
            e03: a1 * g2 + a2 * g1 + c2 * e1 + d2 * f1 - b1 * h2 - b2 * h1 - c1 * e2 - d1 * f2,
            e0123: a1 * h2 + a2 * h1 + b1 * g2 + b2 * g1 + d1 * e2 + d2 * e1 - c1 * f2 - c2 * f1,
        }
    }

    #[inline]
    #[must_use]
    pub const fn transform_point(self, point: Vector3) -> Vector3 {
        /*
            (a + -1*b*e1*e2 + -1*c*e1*e3 + -1*d*e2*e3 + -1*e*e0*e1 + -1*f*e0*e2 + -1*g*e0*e3 + h*e0*e1*e2*e3)
            *
            (1*e1*e2*e3 + x*e0*e3*e2 + y*e0*e1*e3 + z*e0*e2*e1)
            *
            (a + b*e1*e2 + c*e1*e3 + d*e2*e3 + e*e0*e1 + f*e0*e2 + g*e0*e3 + h*e0*e1*e2*e3)

            =

            e0*e1*e2*(-2*a*g + -2*b*h + -2*c*e + -2*d*f + c*c*z + d*d*z + -2*a*c*x + -2*a*d*y + -2*b*d*x + -1*a*a*z + -1*b*b*z + 2*b*c*y)
            e0*e1*e3*(-2*c*h + -2*d*g + 2*a*f + 2*b*e + a*a*y + c*c*y + -2*a*d*z + -2*b*c*z + -2*c*d*x + -1*b*b*y + -1*d*d*y + 2*a*b*x)
            e0*e2*e3*(-2*a*e + -2*d*h + 2*b*f + 2*c*g + b*b*x + c*c*x + -2*b*d*z + -1*a*a*x + -1*d*d*x + 2*a*b*y + 2*a*c*z + 2*c*d*y)
            e1*e2*e3*(a*a + b*b + c*c + d*d) // should always be 1 for a normalised rotor
        */

        let Self {
            s: a,
            e12: b,
            e13: c,
            e23: d,
            e01: e,
            e02: f,
            e03: g,
            e0123: h,
        } = self;
        let Vector3 { x, y, z } = point;

        let e012 = -2.0 * a * g - 2.0 * b * h - 2.0 * c * e - 2.0 * d * f + c * c * z + d * d * z
            - 2.0 * a * c * x
            - 2.0 * a * d * y
            - 2.0 * b * d * x
            - a * a * z
            - b * b * z
            + 2.0 * b * c * y;
        let e013 = -2.0 * c * h - 2.0 * d * g + 2.0 * a * f + 2.0 * b * e + a * a * y + c * c * y
            - 2.0 * a * d * z
            - 2.0 * b * c * z
            - 2.0 * c * d * x
            - b * b * y
            - d * d * y
            + 2.0 * a * b * x;
        let e023 = -2.0 * a * e - 2.0 * d * h + 2.0 * b * f + 2.0 * c * g + b * b * x + c * c * x
            - 2.0 * b * d * z
            - a * a * x
            - d * d * x
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
