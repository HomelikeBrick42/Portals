import rotor;

struct Transform
{
    float s;
    float e12;
    float e13;
    float e23;
    float e01;
    float e02;
    float e03;
    float e0123;

    Rotor rotor_part()
    {
        var rotor : Rotor;
        rotor.s = this.s;
        rotor.e12 = this.e12;
        rotor.e13 = this.e13;
        rotor.e23 = this.e23;
        return rotor;
    }

    Transform inverse()
    {
        var result : Transform;
        result.s = this.s;
        result.e12 = -this.e12;
        result.e13 = -this.e13;
        result.e23 = -this.e23;
        result.e01 = -this.e01;
        result.e02 = -this.e02;
        result.e03 = -this.e03;
        result.e0123 = this.e0123;
        return result;
    }

    Transform next(Transform next)
    {
        return next.after(this);
    }

    Transform after(Transform after)
    {
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

        let a1 = this.s;
        let b1 = this.e12;
        let c1 = this.e13;
        let d1 = this.e23;
        let e1 = this.e01;
        let f1 = this.e02;
        let g1 = this.e03;
        let h1 = this.e0123;
        let a2 = after.s;
        let b2 = after.e12;
        let c2 = after.e13;
        let d2 = after.e23;
        let e2 = after.e01;
        let f2 = after.e02;
        let g2 = after.e03;
        let h2 = after.e0123;

        var result : Transform;
        result.s = a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2;
        result.e12 = a1 * b2 + a2 * b1 + c2 * d1 - c1 * d2;
        result.e13 = a1 * c2 + a2 * c1 + b1 * d2 - b2 * d1;
        result.e23 = a1 * d2 + a2 * d1 + b2 * c1 - b1 * c2;
        result.e01 = a1 * e2 + a2 * e1 + b1 * f2 + c1 * g2 - b2 * f1 - c2 * g1 - d1 * h2 - d2 * h1;
        result.e02 = a1 * f2 + a2 * f1 + b2 * e1 + c1 * h2 + c2 * h1 + d1 * g2 - b1 * e2 - d2 * g1;
        result.e03 = a1 * g2 + a2 * g1 + c2 * e1 + d2 * f1 - b1 * h2 - b2 * h1 - c1 * e2 - d1 * f2;
        result.e0123 = a1 * h2 + a2 * h1 + b1 * g2 + b2 * g1 + d1 * e2 + d2 * e1 - c1 * f2 - c2 * f1;
        return result;
    }

    float3 transform_point(float3 point)
    {
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

        let a = this.s;
        let b = this.e12;
        let c = this.e13;
        let d = this.e23;
        let e = this.e01;
        let f = this.e02;
        let g = this.e03;
        let h = this.e0123;
        let x = point.x;
        let y = point.y;
        let z = point.z;

        let e012 = -2.0 * a * g - 2.0 * b * h - 2.0 * c * e - 2.0 * d * f + c * c * z + d * d * z - 2.0 * a * c * x - 2.0 * a * d * y - 2.0 * b * d * x - a * a * z - b * b * z + 2.0 * b * c * y;
        let e013 = -2.0 * c * h - 2.0 * d * g + 2.0 * a * f + 2.0 * b * e + a * a * y + c * c * y - 2.0 * a * d * z - 2.0 * b * c * z - 2.0 * c * d * x - b * b * y - d * d * y + 2.0 * a * b * x;
        let e023 = -2.0 * a * e - 2.0 * d * h + 2.0 * b * f + 2.0 * c * g + b * b * x + c * c * x - 2.0 * b * d * z - a * a * x - d * d * x + 2.0 * a * b * y + 2.0 * a * c * z + 2.0 * c * d * y;

        return float3(
            -e023,
            e013,
            -e012);
    }
}
