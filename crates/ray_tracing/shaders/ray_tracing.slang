import include.transform;
import include.ray;

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
    float3 ambient_color;
}

struct SceneInfo
{
    Camera camera;
    float aspect;
}

[vk::binding(0, 1)]
ConstantBuffer<SceneInfo> info;

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
    ray.origin = float3(0.0, 0.0, 0.0);
    ray.direction = normalize(forward + up * uv.y + right * uv.x * info.aspect);

    var color = lerp(info.camera.down_sky_color, info.camera.up_sky_color, ray.direction.y * 0.5 + 0.5);

    let sun_direction = normalize(float3(0.4, 1.0, 0.2));
    if (acos(dot(sun_direction, ray.direction)) < info.camera.sun_size)
        color = info.camera.sun_color;

    main_texture.Store(global_index.xy, float4(color, 1.0));
}
