module display;

public float4 screen_to_ndc(float3 pos, float2 screen_size) {
    return float4(
        2. * pos.x / screen_size.x - 1.,
        1. - 2. * pos.y / screen_size.y,
        pos.z,
        1.
    );
}

public float4 linear_to_gamma(float4 color) {
    float3 rgb = color.rgb;
    bool3 cutoff = rgb <= float3(0.0031308);
    float3 lower = rgb * float3(12.92);
    float3 higher = float3(1.055) * pow(rgb, float3(1.0 / 2.4)) - float3(0.055);

    return float4(lerp(higher, lower, float3(cutoff)), color.a);
}

public float4 gamma_to_linear(float4 color) {
    float3 srgb = color.rgb;
    bool3 cutoff = srgb <= float3(0.04045);
    float3 lower = srgb / float3(12.92);
    float3 higher = pow((srgb + float3(0.055)) / float3(1.055), float3(2.4));

    return float4(lerp(higher, lower, float3(cutoff)), color.a);
}
