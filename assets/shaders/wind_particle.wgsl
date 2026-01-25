#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material_color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = material_color;

    // Apply lighting for 3D depth
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let n_dot_l = max(dot(normalize(in.world_normal), light_dir), 0.4);
    color = vec4<f32>(color.rgb * n_dot_l, color.a);

    // Transparency fade along particle length
    // UV.y: 0 = front/head (opaque), 1 = back/tail (transparent)
    let fade = 1.0 - in.uv.y; // 1 at front, 0 at back
    var alpha = pow(fade, 1.5); // Smooth falloff

    // Age-based fade from vertex color alpha
    alpha *= in.color.a;

    color.a = alpha;

    return color;
}
