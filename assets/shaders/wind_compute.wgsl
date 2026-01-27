// Wind particle compute shader
// This shader simulates wind particles on the planet surface

struct Particle {
    position: vec3<f32>,      // Position on sphere surface
    velocity: vec3<f32>,      // Velocity tangent to sphere (unused for now - static particles)
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

// Hash function for pseudo-random values (similar to CPU's random_point_on_sphere approach)
fn hash(n: u32) -> f32 {
    var x = n;
    x = x ^ (x >> 16u);
    x = x * 0x85ebca6bu;
    x = x ^ (x >> 13u);
    x = x * 0xc2b2ae35u;
    x = x ^ (x >> 16u);
    return f32(x) / 4294967296.0;
}

// Hash with two seeds for 2D randomness
fn hash2(seed1: u32, seed2: u32) -> f32 {
    return hash(seed1 * 7919u + seed2 * 31337u);
}

// Generate random point on sphere (similar to CPU's random_point_on_sphere)
// Uses spherical coordinates with uniform distribution
fn random_point_on_sphere(seed: u32) -> vec3<f32> {
    let u = hash2(seed, 1u) * 2.0 - 1.0;  // cos(theta) in range [-1, 1]
    let phi = hash2(seed, 2u) * 2.0 * 3.14159265359;  // azimuthal angle [0, 2π]
    let t = sqrt(1.0 - u * u);  // sin(theta)

    return vec3<f32>(
        t * cos(phi),
        u,
        t * sin(phi)
    );
}

@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let idx = invocation_id.x;
    let particle_count = arrayLength(&particles);

    if (idx >= particle_count) {
        return;
    }

    // Generate RANDOM position on sphere (not Fibonacci - truly random!)
    let direction = random_point_on_sphere(idx);
    let sphere_radius = uniforms.planet_radius + uniforms.particle_height_offset;

    particles[idx].position = direction * sphere_radius;
    particles[idx].velocity = vec3<f32>(0.0, 0.0, 0.0);  // Static for now

    // ALL particles have SAME lifetime (5 seconds)
    particles[idx].lifetime = 5.0;

    // Stagger initial ages (0 to full lifetime) so particles spawn/despawn continuously
    // This creates a rolling spawn/despawn effect
    particles[idx].age = hash2(idx, 4u) * particles[idx].lifetime;
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
        // Use total_time + idx for unique seed each respawn (changes over time!)
        let time_seed = u32(uniforms.total_time * 1000.0) + idx;

        // Generate NEW random position (different every respawn!)
        let direction = random_point_on_sphere(time_seed);
        let sphere_radius = uniforms.planet_radius + uniforms.particle_height_offset;
        particle.position = direction * sphere_radius;

        // Keep SAME lifetime (5 seconds) - all particles have same lifecycle duration
        particle.lifetime = 5.0;

        particle.age = 0.0;
        particle.velocity = vec3<f32>(0.0, 0.0, 0.0);  // Static for now - no wind movement yet
    }

    // TODO: Wind velocity calculation will go here
    // For now, particles stay static (no movement)

    // Keep particle on sphere surface (in case of any floating point drift)
    let sphere_radius = uniforms.planet_radius + uniforms.particle_height_offset;
    particle.position = normalize(particle.position) * sphere_radius;

    particles[idx] = particle;
}
