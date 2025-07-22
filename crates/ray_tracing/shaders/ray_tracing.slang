import include.rotor;
import include.transform;
import include.ray;
import include.plane;

[vk::binding(0, 0)]
[format("rgba32f")]
WTexture2D main_texture;

struct Camera
{
    Transform transform;
    float3 up_sky_color;
    float3 down_sky_color;
    float sun_size;
    float3 sun_color;
    float3 sun_light_color;
    float3 sun_direction;
    float3 ambient_color;
}

struct SceneInfo
{
    Camera camera;
    float aspect;
    uint32_t plane_count;
}

[vk::binding(0, 1)]
ConstantBuffer<SceneInfo> info;

[vk::binding(0, 2)]
StructuredBuffer<Plane> planes;

[shader("compute")]
[numthreads(16, 16, 1)]
void ray_trace(uint3 global_index: SV_DispatchThreadID)
{
    var width : uint;
    var height : uint;
    main_texture.GetDimensions(width, height);

    if (global_index.x >= width || global_index.y >= height)
        return;

    let uv = (float2(global_index.xy) / float2(width, height)) * 2.0 - 1.0;

    let forward = float3(1.0, 0.0, 0.0);
    let up = float3(0.0, 1.0, 0.0);
    let right = float3(0.0, 0.0, 1.0);

    var ray : Ray;
    // TODO: make optimised functions for getting position/basis axes
    ray.origin = info.camera.transform.transform_point(float3(0.0, 0.0, 0.0));
    ray.direction = normalize(info.camera.transform.rotor_part().rotate(forward + up * uv.y + right * uv.x * info.aspect));

    let hit = trace_ray(ray);

    var color = lerp(info.camera.down_sky_color, info.camera.up_sky_color, ray.direction.y * 0.5 + 0.5);

    let sun_direction = normalize(info.camera.sun_direction);
    if (acos(dot(sun_direction, ray.direction)) < info.camera.sun_size)
        color = info.camera.sun_color;

    if (hit.hasValue)
    {
        let hit = hit.value;

        var sun_ray : Ray;
        sun_ray.origin = hit.position;
        sun_ray.direction = sun_direction;

        let sun_hit = trace_ray(sun_ray);
        let sun_light = float(!sun_hit.hasValue) * info.camera.sun_light_color * max(dot(hit.normal, sun_direction), 0.0);
        color = hit.color * clamp(sun_light + info.camera.ambient_color, 0.0, 1.0);
    }

    main_texture.Store(global_index.xy, float4(color, 1.0));
}

Optional<Hit> trace_ray(inout Ray ray)
{
    var result_hit = intersect_scene(ray);
    for (var i = 0; i < 10; i++)
    {
        if (!result_hit.hasValue)
            break;
        let hit = result_hit.value;
        if (!hit.hit_plane.hasValue)
            break;
        let plane = planes[hit.hit_plane.value];

        var other_index = uint32_t.maxValue;
        var flip = false;
        if (hit.front)
        {
            other_index = plane.front_portal.other_index;
            flip = plane.front_portal.flip != 0;
        }
        else
        {
            other_index = plane.back_portal.other_index;
            flip = plane.back_portal.flip != 0;
        }
        if (other_index == uint32_t.maxValue)
            break;

        let other_plane = planes[other_index];
        let transform = other_plane.transform.then(plane.transform.inverse());
        ray.origin = transform.transform_point(ray.origin);
        ray.direction = transform.rotor_part().rotate(ray.direction);
        if (flip)
            ray.direction = -ray.direction;

        result_hit = intersect_scene(ray);
    }
    return result_hit;
}

Optional<Hit> intersect_scene(Ray ray)
{
    var closest_hit : Optional<Hit> = none;
    for (uint32_t i = 0; i < info.plane_count; i++)
    {
        let hit = planes[i].Intersect(ray);
        if (hit.hasValue && hit.value.distance > 0.001 && (!closest_hit.hasValue || hit.value.distance < closest_hit.value.distance))
        {
            var hit = hit.value;
            hit.hit_plane = i;
            closest_hit = hit;
        }
    }
    return closest_hit;
}
