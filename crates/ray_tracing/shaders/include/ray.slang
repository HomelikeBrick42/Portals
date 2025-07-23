struct Ray
{
    float3 origin;
    float3 direction;
}

struct Hit
{
    float distance;
    float3 position;
    float3 normal;
    float3 color;
    float3 emissive_color;
    bool front;

    Optional<uint32_t> hit_plane;
}
