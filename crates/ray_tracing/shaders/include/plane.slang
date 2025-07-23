import transform;
import ray;

struct Plane
{
    Transform transform;
    float3 color;
    float width;
    float height;
    uint32_t checker_count_x;
    uint32_t checker_count_z;
    float checker_darkness;
    PortalConnection front_portal;
    PortalConnection back_portal;

    Optional<Hit> Intersect(Ray ray)
    {
        let inverse_transform = this.transform.inverse();
        let origin = inverse_transform.transform_point(ray.origin);
        let direction = inverse_transform.rotor_part().rotate(ray.direction);

        if (sign(origin.y) == sign(direction.y) || abs(direction.y) < 0.001)
            return none;

        var hit : Hit;

        hit.distance = abs(origin.y / direction.y);
        hit.position = ray.origin + ray.direction * hit.distance;
        hit.normal = normalize(this.transform.rotor_part().rotate(float3(0.0, origin.y, 0.0)));
        hit.color = this.color;
        hit.front = direction.y < 0.0;

        let local_pos = origin.xz + direction.xz * hit.distance;
        if (local_pos.x < this.width * -0.5 || local_pos.y < this.height * -0.5 || local_pos.x > this.width * 0.5 || local_pos.y > this.height * 0.5)
            return none;

        let cell = uint2((local_pos / float2(this.width, this.height) + 0.5) * float2(this.checker_count_x, this.checker_count_z));
        if ((cell.x + cell.y) % 2 == 1)
            hit.color *= this.checker_darkness;

        return hit;
    }
}

struct PortalConnection
{
    /// uint32_t.maxValue is no connection
    uint32_t other_index;
    // uint32_t flip;
}
