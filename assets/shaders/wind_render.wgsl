// Wind particle rendering shader
// Renders particles as billboarded quads

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

@group(0) @binding(0) var<storage, read> particles: array<Particle>;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@group(1) @binding(0) var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let particle = particles[in.instance_index];

    // Billboard quad vertices (in view space)
    let size = 0.5;
    var offset: vec2<f32>;
    switch (in.vertex_index) {
        case 0u: { offset = vec2<f32>(-size, -size); }
        case 1u: { offset = vec2<f32>(size, -size); }
        case 2u: { offset = vec2<f32>(-size, size); }
        case 3u: { offset = vec2<f32>(size, -size); }
        case 4u: { offset = vec2<f32>(size, size); }
        case 5u: { offset = vec2<f32>(-size, size); }
        default: { offset = vec2<f32>(0.0, 0.0); }
    }

    // TODO: Proper billboard transformation
    // For now, just render at particle position
    let world_pos = vec4<f32>(particle.position, 1.0);
    out.clip_position = view_proj * world_pos;

    // Calculate fade based on age
    let fade_in_time = 0.2;
    let fade_out_time = 0.5;
    var alpha = 1.0;

    if (particle.age < fade_in_time) {
        alpha = particle.age / fade_in_time;
    } else if (particle.age > particle.lifetime - fade_out_time) {
        alpha = (particle.lifetime - particle.age) / fade_out_time;
    }

    out.color = vec4<f32>(1.0, 1.0, 0.8, alpha);

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
