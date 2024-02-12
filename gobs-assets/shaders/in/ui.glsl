
vec4 screen_to_ndc(vec3 pos, vec2 screen_size) {
    return vec4(
        2. * pos.x / screen_size.x - 1.,
        1. - 2. * pos.y / screen_size.y,
        pos.z,
        1.
    );
}

vec4 linear_to_gamma(vec4 rgba) {
    vec3 rgb = rgba.xyz;
    bvec3 cutoff = lessThan(rgb, vec3(0.0031308));
    vec3 lower = rgb * vec3(12.92);
    vec3 higher = vec3(1.055) * pow(rgb, vec3(1.0 / 2.4)) - vec3(0.055);

    return vec4(mix(higher, lower, vec3(cutoff)), rgba.a);
}
