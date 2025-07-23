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
    float up_sky_intensity;
    float3 down_sky_color;
    float down_sky_intensity;
    float3 sun_color;
    float sun_intensity;
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

    let uv = ((float2(global_index.xy) + float2(random_value(state), random_value(state))) / float2(width, height)) * 2.0 - 1.0;

    let forward = float3(1.0, 0.0, 0.0);
    let up = float3(0.0, 1.0, 0.0);
    let right = float3(0.0, 0.0, 1.0);

    var ray : Ray;
    // TODO: make optimised functions for getting position/basis axes
    ray.origin = info.camera.transform.transform_point(float3(0.0, 0.0, 0.0));
    ray.direction = normalize(info.camera.transform.rotor_part().rotate(forward + up * uv.y + right * uv.x * info.aspect));

    var color = ray_color(state, ray);

    let old_color = main_texture.Load(global_index.xy).rgb;
    main_texture.Store(global_index.xy, float4(old_color + (color - old_color) / (info.accumulated_frames + 1), 1.0));
}

float3 ray_color(inout uint32_t state, Ray ray)
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

            let emitted_light = float3(0.0);
            incoming_light += emitted_light * ray_color;
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

float3 skybox(Ray ray)
{
    let up_color = info.camera.up_sky_color * info.camera.up_sky_intensity;
    let down_color = info.camera.down_sky_color * info.camera.down_sky_intensity;
    var color = lerp(down_color, up_color, ray.direction.y * 0.5 + 0.5);
    if (acos(dot(info.camera.sun_direction, ray.direction)) < info.camera.sun_size)
        color = info.camera.sun_color * info.camera.sun_intensity;
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
