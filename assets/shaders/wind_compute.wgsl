// Wind particle compute shader
// This shader simulates wind particles on the planet surface

struct Particle {
    position: vec3<f32>,      // Position on sphere surface
    velocity: vec3<f32>,      // Velocity tangent to sphere
    age: f32,                 // Current age in seconds
    lifetime: f32,            // Max lifetime before respawn
}

struct Uniforms {
    planet_radius: f32,
    particle_height_offset: f32,
    delta_time: f32,
    total_time: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

// Fibonacci sphere distribution for uniform points on a sphere
fn fibonacci_sphere(i: u32, n: u32) -> vec3<f32> {
    let phi = 3.14159265359 * (sqrt(5.0) - 1.0); // Golden angle
    let y = 1.0 - (f32(i) / f32(n - 1u)) * 2.0;  // Y from 1 to -1
    let radius = sqrt(1.0 - y * y);
    let theta = phi * f32(i);

    let x = cos(theta) * radius;
    let z = sin(theta) * radius;

    return normalize(vec3<f32>(x, y, z));
}

// Simple hash function for randomness
fn hash(n: u32) -> f32 {
    var x = n;
    x = x ^ (x >> 16u);
    x = x * 0x85ebca6bu;
    x = x ^ (x >> 13u);
    x = x * 0xc2b2ae35u;
    x = x ^ (x >> 16u);
    return f32(x) / 4294967296.0;
}

@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let idx = invocation_id.x;
    let particle_count = arrayLength(&particles);

    if (idx >= particle_count) {
        return;
    }

    // Distribute particles uniformly on sphere using Fibonacci sphere
    let direction = fibonacci_sphere(idx, particle_count);
    let sphere_radius = uniforms.planet_radius + uniforms.particle_height_offset;

    particles[idx].position = direction * sphere_radius;
    particles[idx].velocity = vec3<f32>(0.0, 0.0, 0.0);

    // Stagger particle ages so they don't all spawn at once
    particles[idx].age = hash(idx) * 10.0;
    particles[idx].lifetime = 10.0;
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let idx = invocation_id.x;
    let particle_count = arrayLength(&particles);

    if (idx >= particle_count) {
        return;
    }

    var particle = particles[idx];

    // Update age
    particle.age += uniforms.delta_time;

    // Respawn if lifetime exceeded
    if (particle.age >= particle.lifetime) {
        let direction = fibonacci_sphere(idx, particle_count);
        let sphere_radius = uniforms.planet_radius + uniforms.particle_height_offset;
        particle.position = direction * sphere_radius;
        particle.velocity = vec3<f32>(0.0, 0.0, 0.0);
        particle.age = 0.0;
    }

    // Update position based on velocity
    particle.position += particle.velocity * uniforms.delta_time;

    // Keep particle on sphere surface
    let sphere_radius = uniforms.planet_radius + uniforms.particle_height_offset;
    particle.position = normalize(particle.position) * sphere_radius;

    particles[idx] = particle;
}
