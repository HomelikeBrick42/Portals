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
    bool front;
}
