#import bevy_pbr::forward_io::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material_color: vec4<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = material_color;

    // Apply lighting FIRST (before transparency calculations)
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let n_dot_l = max(dot(normalize(in.world_normal), light_dir), 0.6);
    color = vec4<f32>(color.rgb * n_dot_l, color.a);

    // Create a tapered shape: wide at front (left), narrow at back (right)
    let max_radius = 0.2;  // Radius at the front (left)
    let min_radius = 0.02; // Radius at the back (right)

    // Define front and back points of the capsule centerline
    let front_point = vec2<f32>(0.15, 0.5);  // Front circle center (left)
    let back_point = vec2<f32>(0.85, 0.5);   // Back end of line (right)

    // Vector from back to front
    let line_vec = front_point - back_point;
    let line_length = length(line_vec);
    let line_dir = line_vec / line_length;

    // Point on the line closest to current UV
    let to_uv = in.uv - back_point;
    let projection = clamp(dot(to_uv, line_dir), 0.0, line_length);
    let closest_point = back_point + line_dir * projection;

    // Calculate varying radius: interpolate from max (left) to min (right) based on u.x
    let radius_t = 1.0 - in.uv.x; // 1.0 at left, 0.0 at right
    let particle_radius = mix(min_radius, max_radius, radius_t);

    // Distance from UV to the capsule centerline
    let dist = length(in.uv - closest_point);

    // Create smooth edge with varying radius
    var alpha = 1.0 - smoothstep(particle_radius - 0.05, particle_radius, dist);

    // Exponential transparency gradient: left (u=0) = opaque, right (u=1) = transparent
    // Using pow for much more dramatic fade
    let fade = 1.0 - in.uv.x;
    alpha *= fade * fade * fade;  // Cubic falloff for very dramatic fade

    // Age-based fade from vertex color alpha (set per-particle from Rust code)
    // Vertex color alpha encodes the lifetime fade (0=just spawned/about to despawn, 1=mid-life)
    alpha *= in.color.a;

    color.a = alpha;

    return color;
}
