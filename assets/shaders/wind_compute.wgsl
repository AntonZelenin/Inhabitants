// Wind particle compute shader
// This shader will simulate wind particles on the planet surface

// Placeholder - will be implemented in next steps
@compute @workgroup_size(64)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // TODO: Initialize particles
}

@compute @workgroup_size(64)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // TODO: Update particles
}
