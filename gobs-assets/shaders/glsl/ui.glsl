
vec4 screen_to_ndc(vec3 pos, vec2 screen_size) {
    return vec4(
        2. * pos.x / screen_size.x - 1.,
        1. - 2. * pos.y / screen_size.y,
        pos.z,
        1.
    );
}

vec4 linear_to_gamma(vec4 color) {
    vec3 rgb = color.rgb;
    bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
    vec3 lower = rgb * vec3(12.92);
    vec3 higher = vec3(1.055) * pow(rgb, vec3(1.0 / 2.4)) - vec3(0.055);

    return vec4(mix(higher, lower, vec3(cutoff)), color.a);
}

vec4 gamma_to_linear(vec4 color) {
    vec3 srgb = color.rgb;
    bvec3 cutoff = lessThan(srgb, vec3(0.04045));
    vec3 lower = srgb / vec3(12.92);
    vec3 higher = pow((srgb + vec3(0.055)) / vec3(1.055), vec3(2.4));

    return vec4(mix(higher, lower, vec3(cutoff)), color.a);
}
