import transform;
import ray;

struct Plane
{
    Transform transform;
    float3 color;
    float width;
    float height;
    uint32_t checker_count;
    float checker_darkness;

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
        hit.front = origin.y > 0.0;

        let local_pos = origin.xz + direction.xz * hit.distance;
        if (local_pos.x < this.width * -0.5 || local_pos.y < this.height * -0.5 || local_pos.x > this.width * 0.5 || local_pos.y > this.height * 0.5)
            return none;

        let cell = uint2((local_pos / float2(this.width, this.height) + 0.5) * this.checker_count);
        if ((cell.x + cell.y) % 2 == 1)
            hit.color *= this.checker_darkness;

        return hit;
    }
}
