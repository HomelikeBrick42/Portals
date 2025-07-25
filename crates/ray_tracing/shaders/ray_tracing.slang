import include.rotor;
import include.transform;
import include.ray;
import include.plane;
import include.random;

[vk::binding(0, 0)]
[format("rgba32f")]
RWTexture2D main_texture;

struct Camera
{
    Transform transform;
    float3 up_sky_color;
    float3 down_sky_color;
    float3 sun_color;
    float3 sun_direction;
    float sun_size;
    uint32_t recursive_portal_count;
    uint32_t max_bounces;
}

struct SceneInfo
{
    Camera camera;
    float aspect;
    uint32_t accumulated_frames;
    uint32_t random_seed;
    uint32_t render_type;
    uint32_t samples_per_pixel;
    uint32_t antialiasing;
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

    var state = info.random_seed + global_index.x * 90359791 + global_index.y * 29705237;

    let forward = float3(1.0, 0.0, 0.0);
    let up = float3(0.0, 1.0, 0.0);
    let right = float3(0.0, 0.0, 1.0);

    var color = float3(0.0, 0.0, 0.0);
    for (var i = 0u; i < info.samples_per_pixel; i++)
    {
        var uv_nudge = float2(0.5);
        if (info.antialiasing != 0)
            uv_nudge = float2(random_value(state), random_value(state));
        let uv = ((float2(global_index.xy) + uv_nudge) / float2(width, height)) * 2.0 - 1.0;

        var ray : Ray;
        // TODO: make optimised functions for getting position/basis axes
        ray.origin = info.camera.transform.transform_point(float3(0.0, 0.0, 0.0));
        ray.direction = normalize(info.camera.transform.rotor_part().rotate(forward + up * uv.y + right * uv.x * info.aspect));

        switch (info.render_type)
        {
        case 0:
            color += ray_color_unlit(state, ray);
            break;
        case 1:
            color += ray_color_lit(state, ray);
            break;
        }
    }
    color /= info.samples_per_pixel;

    var old_color = main_texture.Load(global_index.xy).rgb;
    if (info.accumulated_frames == 0)
        old_color = float3(0.0);
    main_texture.Store(global_index.xy, float4(old_color + (color - old_color) / (info.accumulated_frames + 1), 1.0));
}

float3 ray_color_lit(inout uint32_t state, Ray ray)
{
    var incoming_light = float3(0.0);
    var ray_color = float3(1.0);

    for (var i = 0u; i < info.camera.max_bounces; i++)
    {
        let hit = trace_ray(ray);
        if (hit.hasValue)
        {
            let hit = hit.value;

            ray.origin = hit.position + hit.normal * 0.001;
            ray.direction = normalize(hit.normal + random_direction(state) * 0.999);

            incoming_light += hit.emissive_color * ray_color;
            ray_color *= hit.color;
        }
        else
        {
            incoming_light += skybox(ray) * ray_color;
            break;
        }
    }

    return incoming_light;
}

float3 ray_color_unlit(inout uint32_t state, Ray ray)
{
    let hit = trace_ray(ray);
    if (hit.hasValue)
    {
        let hit = hit.value;
        return hit.color + hit.emissive_color;
    }
    else
    {
        return skybox(ray);
    }
}

float3 skybox(Ray ray)
{
    var color = lerp(info.camera.down_sky_color, info.camera.up_sky_color, ray.direction.y * 0.5 + 0.5);
    if (acos(dot(info.camera.sun_direction, ray.direction)) < info.camera.sun_size)
        color = info.camera.sun_color;
    return color;
}

Optional<Hit> trace_ray(inout Ray ray)
{
    var result_hit = intersect_scene(ray);
    for (var i = 0u; i < info.camera.recursive_portal_count; i++)
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
            // flip = plane.front_portal.flip != 0;
        }
        else
        {
            other_index = plane.back_portal.other_index;
            // flip = plane.back_portal.flip != 0;
        }
        if (other_index == uint32_t.maxValue)
            break;

        let other_plane = planes[other_index];
        let transform = other_plane.transform.then(plane.transform.inverse());

        var nudge = hit.normal * 0.001;
        if (flip)
            ray.direction = reflect(ray.direction, hit.normal);
        else
            nudge = -nudge;
        ray.origin = transform.transform_point(hit.position + nudge);
        ray.direction = transform.rotor_part().rotate(ray.direction);

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
        if (hit.hasValue && (!closest_hit.hasValue || hit.value.distance < closest_hit.value.distance))
        {
            var hit = hit.value;
            hit.hit_plane = i;
            closest_hit = hit;
        }
    }
    return closest_hit;
}
