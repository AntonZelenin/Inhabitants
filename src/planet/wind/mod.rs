pub mod components;
pub mod systems;

use bevy::{
    prelude::*,
    pbr::{ExtendedMaterial, MaterialExtension},
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{RenderGraph, RenderLabel},
        render_resource::{binding_types::*, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderStartup, RenderSystems,
    },
};
use std::borrow::Cow;
use bevy::shader::ShaderRef;

/// Shader asset path for wind particle compute shader
const WIND_COMPUTE_SHADER: &str = "shaders/wind_compute.wgsl";
const WIND_PARTICLE_SHADER: &str = "shaders/wind_particle.wgsl";
const WIND_RENDER_GPU_SHADER: &str = "shaders/wind_render_gpu.wgsl";

/// Workgroup size for compute shader (must match shader)
const WORKGROUP_SIZE: u32 = 64;

// Material extension for wind particles with time uniforms
// Extension bindings automatically go to bind group 2
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WindParticleMaterial {
    // StandardMaterial uses bindings 0..=12 in Bevy 0.17; extend at 13.
    #[uniform(13)]
    pub time_uniforms: WindTimeUniforms,
}

#[derive(Debug, Clone, ShaderType, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct WindTimeUniforms {
    pub time_now: f32,
    pub lifetime: f32,
    pub fade_in: f32,
    pub fade_out: f32,
}

impl MaterialExtension for WindParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        WIND_PARTICLE_SHADER.into()
    }
}

pub type WindMaterial = ExtendedMaterial<StandardMaterial, WindParticleMaterial>;

// Particle data structure matching the shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
struct GpuParticle {
    position: [f32; 3],
    _padding1: f32,
    velocity: [f32; 3],
    _padding2: f32,
    age: f32,
    lifetime: f32,
    _padding3: [f32; 2],
}

// Uniforms for compute shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
struct WindUniforms {
    planet_radius: f32,
    particle_height_offset: f32,
    delta_time: f32,
    total_time: f32,
}

// Resource to pass planet settings to render world
#[derive(Resource, Clone, ExtractResource)]
pub struct WindParticleSettings {
    pub planet_radius: f32,
    pub particle_height_offset: f32,
    pub particle_count: usize,
    pub enabled: bool,
}

impl Default for WindParticleSettings {
    fn default() -> Self {
        Self {
            planet_radius: 50.0,
            particle_height_offset: 2.0,
            particle_count: 500,
            enabled: true,
        }
    }
}

pub struct ComputeWindPlugin;

impl Plugin for ComputeWindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<WindMaterial>::default())
            .init_resource::<WindParticleSettings>()
            .add_plugins(ExtractResourcePlugin::<WindParticleSettings>::default())
            .add_systems(Update, systems::update_wind_settings)
            .add_systems(Update, systems::handle_wind_tab_events)
            .add_systems(Update, systems::spawn_wind_particles)
            .add_systems(Update, systems::update_particle_with_movement);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(RenderStartup, init_wind_pipeline)
            .add_systems(RenderStartup, init_wind_render_pipeline)
            .add_systems(Render, prepare_wind_resources.in_set(RenderSystems::PrepareResources));

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(WindComputeLabel, WindComputeNode::default());
        render_graph.add_node(WindRenderLabel, WindRenderNode::default());
        render_graph.add_node_edge(WindComputeLabel, WindRenderLabel);
        render_graph.add_node_edge(WindRenderLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<WindComputePipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WindComputeLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WindRenderLabel;

#[derive(Resource)]
struct WindComputePipeline {
    bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

#[derive(Resource)]
struct WindGpuBuffers {
    particle_buffer: Buffer,
    uniform_buffer: Buffer,
    time_uniform_buffer: Buffer,
    bind_group: BindGroup,
    render_bind_group: BindGroup,
}

#[derive(Resource)]
struct WindRenderPipeline {
    pipeline: CachedRenderPipelineId,
    bind_group_layout: BindGroupLayout,
}

impl FromWorld for WindComputePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        // Create bind group layout
        let bind_group_layout = render_device.create_bind_group_layout(
            "WindParticleBindGroupLayout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    storage_buffer::<GpuParticle>(false),
                    uniform_buffer::<WindUniforms>(false),
                ),
            ),
        );

        Self {
            bind_group_layout,
            init_pipeline: CachedComputePipelineId::INVALID,
            update_pipeline: CachedComputePipelineId::INVALID,
        }
    }
}

fn init_wind_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<WindComputePipeline>,
) {
    let shader = asset_server.load(WIND_COMPUTE_SHADER);

    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::from("wind_particle_init_pipeline")),
        layout: vec![pipeline.bind_group_layout.clone()],
        push_constant_ranges: vec![],
        shader: shader.clone(),
        shader_defs: vec![],
        entry_point: Some(Cow::from("init")),
        zero_initialize_workgroup_memory: false,
    });

    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::from("wind_particle_update_pipeline")),
        layout: vec![pipeline.bind_group_layout.clone()],
        push_constant_ranges: vec![],
        shader,
        shader_defs: vec![],
        entry_point: Some(Cow::from("update")),
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(WindComputePipeline {
        bind_group_layout: pipeline.bind_group_layout.clone(),
        init_pipeline,
        update_pipeline,
    });
}

fn init_wind_render_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
    render_device: Res<RenderDevice>,
) {
    let shader = asset_server.load(WIND_RENDER_GPU_SHADER);

    // Create bind group layout for particle data (group 0)
    let particle_bind_group_layout = render_device.create_bind_group_layout(
        "WindRenderBindGroupLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                storage_buffer_read_only::<GpuParticle>(false),
                uniform_buffer::<WindUniforms>(true),
                uniform_buffer::<WindTimeUniforms>(true),
            ),
        ),
    );

    // Get view bind group layout (group 1) from Bevy's render resources
    // We need to get this from the world, but for now we'll create a compatible one
    let view_bind_group_layout = render_device.create_bind_group_layout(
        "WindViewBindGroupLayout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<bevy::render::view::ViewUniform>(true),
            ),
        ),
    );

    let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(Cow::from("wind_particle_render_pipeline")),
        layout: vec![particle_bind_group_layout.clone(), view_bind_group_layout.clone()],
        push_constant_ranges: vec![],
        vertex: VertexState {
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Some(Cow::from("vertex")),
            buffers: vec![],
        },
        primitive: PrimitiveState {
            topology: PrimitiveTopology::PointList,
            ..default()
        },
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Greater,
            stencil: default(),
            bias: default(),
        }),
        multisample: MultisampleState::default(),
        fragment: Some(FragmentState {
            shader,
            shader_defs: vec![],
            entry_point: Some(Cow::from("fragment")),
            targets: vec![Some(ColorTargetState {
                format: TextureFormat::Rgba16Float,
                blend: Some(BlendState::ALPHA_BLENDING),
                write_mask: ColorWrites::ALL,
            })],
        }),
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(WindRenderPipeline {
        pipeline: pipeline_id,
        bind_group_layout: particle_bind_group_layout,
    });
}

fn prepare_wind_resources(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline: Res<WindComputePipeline>,
    render_pipeline: Option<Res<WindRenderPipeline>>,
    settings: Res<WindParticleSettings>,
    time: Res<Time>,
    buffers: Option<Res<WindGpuBuffers>>,
) {
    // Create buffers on first run
    if buffers.is_none() {
        let particle_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("wind_particle_buffer"),
            size: (settings.particle_count * std::mem::size_of::<GpuParticle>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("wind_uniform_buffer"),
            size: std::mem::size_of::<WindUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let time_uniform_buffer = render_device.create_buffer(&BufferDescriptor {
            label: Some("wind_time_uniform_buffer"),
            size: std::mem::size_of::<WindTimeUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = render_device.create_bind_group(
            "WindParticleBindGroup",
            &pipeline.bind_group_layout,
            &BindGroupEntries::sequential((
                particle_buffer.as_entire_buffer_binding(),
                uniform_buffer.as_entire_buffer_binding(),
            )),
        );

        // Create render bind group if render pipeline exists
        let render_bind_group = if let Some(ref render_pipeline) = render_pipeline {
            render_device.create_bind_group(
                "WindRenderBindGroup",
                &render_pipeline.bind_group_layout,
                &BindGroupEntries::sequential((
                    particle_buffer.as_entire_buffer_binding(),
                    uniform_buffer.as_entire_buffer_binding(),
                    time_uniform_buffer.as_entire_buffer_binding(),
                )),
            )
        } else {
            // Temporary empty bind group
            render_device.create_bind_group(
                "WindRenderBindGroupTemp",
                &pipeline.bind_group_layout,
                &BindGroupEntries::sequential((
                    particle_buffer.as_entire_buffer_binding(),
                    uniform_buffer.as_entire_buffer_binding(),
                )),
            )
        };

        commands.insert_resource(WindGpuBuffers {
            particle_buffer,
            uniform_buffer,
            time_uniform_buffer,
            bind_group,
            render_bind_group,
        });
    }

    // Update uniforms
    if let Some(buffers) = buffers {
        let uniforms = WindUniforms {
            planet_radius: settings.planet_radius,
            particle_height_offset: settings.particle_height_offset,
            delta_time: time.delta_secs(),
            total_time: time.elapsed_secs(),
        };

        render_queue.write_buffer(
            &buffers.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        let time_uniforms = WindTimeUniforms {
            time_now: time.elapsed_secs(),
            lifetime: 5.0,
            fade_in: 0.3,
            fade_out: 0.5,
        };

        render_queue.write_buffer(
            &buffers.time_uniform_buffer,
            0,
            bytemuck::cast_slice(&[time_uniforms]),
        );
    }
}

enum WindComputeState {
    Loading,
    Init,
    Update,
}

#[derive(Default)]
struct WindComputeNode {
    state: Option<WindComputeState>,
}

impl Default for WindComputeState {
    fn default() -> Self {
        Self::Loading
    }
}

impl bevy::render::render_graph::Node for WindComputeNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<WindComputePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        if self.state.is_none() {
            self.state = Some(WindComputeState::Loading);
        }

        // Transition states based on pipeline loading
        match self.state.as_ref().unwrap() {
            WindComputeState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = Some(WindComputeState::Init);
                }
            }
            WindComputeState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = Some(WindComputeState::Update);
                }
            }
            WindComputeState::Update => {
                // Stay in update state
            }
        }
    }

    fn run(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let Some(state) = &self.state else {
            return Ok(());
        };

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<WindComputePipeline>();
        let settings = world.resource::<WindParticleSettings>();
        let Some(buffers) = world.get_resource::<WindGpuBuffers>() else {
            return Ok(());
        };

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("wind_particle_compute_pass"),
                timestamp_writes: None,
            });

        let particle_count = settings.particle_count as u32;

        match state {
            WindComputeState::Loading => {}
            WindComputeState::Init => {
                let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline) else {
                    return Ok(());
                };
                pass.set_bind_group(0, &buffers.bind_group, &[]);
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups((particle_count + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1);
            }
            WindComputeState::Update => {
                let Some(update_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.update_pipeline) else {
                    return Ok(());
                };
                pass.set_bind_group(0, &buffers.bind_group, &[]);
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups((particle_count + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1);
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct WindRenderNode;

impl bevy::render::render_graph::Node for WindRenderNode {
    fn run(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        // TODO: Implement proper ViewNode integration to access camera render targets
        // For now, skip GPU rendering to prevent encoder validation errors
        // The compute shader still runs and updates particle positions
        Ok(())
    }
}
