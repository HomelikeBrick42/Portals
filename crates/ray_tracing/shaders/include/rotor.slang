struct Rotor
{
    float s;
    float e12;
    float e13;
    float e23;

    Rotor then(Rotor then)
    {
        return then.after(this);
    }

    Rotor after(Rotor after)
    {
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

        let a1 = this.s;
        let b1 = this.e12;
        let c1 = this.e13;
        let d1 = this.e23;
        let a2 = after.s;
        let b2 = after.e12;
        let c2 = after.e13;
        let d2 = after.e23;

        var result : Rotor;
        result.s = a1 * a2 - b1 * b2 - c1 * c2 - d1 * d2;
        result.e12 = a1 * b2 + a2 * b1 + c2 * d1 - c1 * d2;
        result.e13 = a1 * c2 + a2 * c1 + b1 * d2 - b2 * d1;
        result.e23 = a1 * d2 + a2 * d1 + b2 * c1 - b1 * c2;
        return result;
    }

    float3 rotate(float3 point)
    {
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

        let a = this.s;
        let b = this.e12;
        let c = this.e13;
        let d = this.e23;
        let x = point.x;
        let y = point.y;
        let z = point.z;

        let e012 =
            c * c * z + d * d * z - a * c * x - a * d * y - b * d * x - a * a * z - b * b * z + 2.0 * b * c * y;
        let e013 =
            a * a * y + c * c * y - a * d * z - b * c * z - c * d * x - b * b * y - d * d * y + 2.0 * a * b * x;
        let e023 = b * b * x + c * c * x - b * d * z - a * a * x - d * d * x + 2.0 * a * b * y + 2.0 * a * c * z + 2.0 * c * d * y;

        return float3(
            -e023,
            e013,
            -e012);
    }
}
