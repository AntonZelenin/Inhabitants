#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_bindings::mesh

struct TimeUniforms {
    time_now: f32,
    lifetime: f32,
    fade_in: f32,
    fade_out: f32,
}

// Extension materials use a separate bind group (group 2 for material extensions)
@group(#{MATERIAL_BIND_GROUP}) @binding(13)
var<uniform> time_uniforms: TimeUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Start with emissive yellow color
    var color = vec4<f32>(1.0, 1.0, 0.8, 1.0);

    // Apply lighting for 3D depth
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let n_dot_l = max(dot(normalize(in.world_normal), light_dir), 0.4);
    color = vec4<f32>(color.rgb * n_dot_l, 1.0);

    // Extract spawn_time from vertex color.r (we pack it there)
    let spawn_time = in.color.r;

    // Calculate age: time_now - spawn_time
    let age = time_uniforms.time_now - spawn_time;

    // Calculate time until death: lifetime - age
    let time_until_death = time_uniforms.lifetime - age;

    // Calculate alpha based on age (fade in) and time_until_death (fade out)
    let fade_in_alpha = clamp(age / time_uniforms.fade_in, 0.0, 1.0);
    let fade_out_alpha = clamp(time_until_death / time_uniforms.fade_out, 0.0, 1.0);
    let alpha = fade_in_alpha * fade_out_alpha;

    color.a = alpha;

    return color;
}
