import include.transform;

[vk::binding(0, 0)]
[format("rgba32f")]
WTexture2D main_texture;

struct Camera
{
    Transform transform;
    float3 up_sky_color;
    float3 down_sky_color;
    float3 sun_color;
    float3 sun_light_color;
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

    let uv = (float2(global_index.xy) / float2(width, height));

    var color = float3(uv, 0.0);
    main_texture.Store(global_index.xy, float4(color, 1.0));
}
