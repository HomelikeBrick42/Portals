float random_value(inout uint32_t state)
{
    state = state * 747796405u + 2891336453u;
    var result = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    result = (result >> 22u) ^ result;
    return float(result) / 4294967295.0;
}

float random_value_normal_distribution(inout uint32_t state)
{
    let theta = 2.0 * 3.1415926 * random_value(state);
    let rho = sqrt(-2.0 * log(random_value(state)));
    return rho * cos(theta);
}

float3 random_direction(inout uint32_t state)
{
    return normalize(float3(
        random_value_normal_distribution(state),
        random_value_normal_distribution(state),
        random_value_normal_distribution(state)));
}

float3 random_direction_in_hemisphere(inout uint32_t state, float3 normal)
{
    var direction = random_direction(state);
    direction *= sign(dot(direction, normal));
    return direction;
}
