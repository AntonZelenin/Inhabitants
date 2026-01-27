// GPU-driven wind particle rendering
// Reads particle data from storage buffer and renders as instanced points

struct Particle {
    position: vec3<f32>,
    velocity: vec3<f32>,
    age: f32,
    lifetime: f32,
}

struct Uniforms {
    planet_radius: f32,
    particle_height_offset: f32,
    delta_time: f32,
    total_time: f32,
}

struct TimeUniforms {
    time_now: f32,
    lifetime: f32,
    fade_in: f32,
    fade_out: f32,
}

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;
@group(0) @binding(2) var<uniform> time_uniforms: TimeUniforms;

// View bindings (camera matrices)
#import bevy_render::view::View
@group(1) @binding(0) var<uniform> view: View;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) age: f32,
    @location(2) lifetime: f32,
};

@vertex
fn vertex(@builtin(instance_index) instance_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let particle = particles[instance_index];
    
    // Transform world position to clip space
    out.clip_position = view.clip_from_world * vec4<f32>(particle.position, 1.0);
    out.world_position = particle.position;
    out.age = particle.age;
    out.lifetime = particle.lifetime;
    
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Base color: emissive yellow
    var color = vec3<f32>(1.0, 1.0, 0.8);
    
    // Calculate alpha based on age (fade in) and time_until_death (fade out)
    let age = in.age;
    let time_until_death = in.lifetime - age;
    
    let fade_in_alpha = clamp(age / time_uniforms.fade_in, 0.0, 1.0);
    let fade_out_alpha = clamp(time_until_death / time_uniforms.fade_out, 0.0, 1.0);
    let alpha = fade_in_alpha * fade_out_alpha;
    
    // Apply some basic lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let normal = normalize(in.world_position);
    let n_dot_l = max(dot(normal, light_dir), 0.4);
    color = color * n_dot_l;
    
    return vec4<f32>(color, alpha);
}
