pub mod components;
pub mod systems;

use bevy::{
    prelude::*,
    render::{
        render_graph::{RenderGraph, RenderLabel},
        render_resource::*,
        renderer::RenderContext,
        RenderApp, RenderStartup,
    },
};
use std::borrow::Cow;

/// Shader asset path for wind particle compute shader
const WIND_COMPUTE_SHADER: &str = "shaders/wind_compute.wgsl";

/// Number of particles to simulate
const PARTICLE_COUNT: u32 = 500;

/// Workgroup size for compute shader (must match shader)
const WORKGROUP_SIZE: u32 = 64;

pub struct ComputeWindPlugin;

impl Plugin for ComputeWindPlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(RenderStartup, init_wind_pipeline);

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(WindComputeLabel, WindComputeNode::default());
        render_graph.add_node_edge(WindComputeLabel, bevy::render::graph::CameraDriverLabel);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<WindComputePipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct WindComputeLabel;

#[derive(Resource)]
struct WindComputePipeline {
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for WindComputePipeline {
    fn from_world(_world: &mut World) -> Self {
        // Placeholder - will be properly initialized in init_wind_pipeline
        Self {
            init_pipeline: CachedComputePipelineId::INVALID,
            update_pipeline: CachedComputePipelineId::INVALID,
        }
    }
}

fn init_wind_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let shader = asset_server.load(WIND_COMPUTE_SHADER);

    let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::from("wind_particle_init_pipeline")),
        layout: vec![],
        push_constant_ranges: vec![],
        shader: shader.clone(),
        shader_defs: vec![],
        entry_point: Some(Cow::from("init")),
        zero_initialize_workgroup_memory: false,
    });

    let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::from("wind_particle_update_pipeline")),
        layout: vec![],
        push_constant_ranges: vec![],
        shader,
        shader_defs: vec![],
        entry_point: Some(Cow::from("update")),
        zero_initialize_workgroup_memory: false,
    });

    commands.insert_resource(WindComputePipeline {
        init_pipeline,
        update_pipeline,
    });
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

        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor {
                label: Some("wind_particle_compute_pass"),
                timestamp_writes: None,
            });

        match state {
            WindComputeState::Loading => {}
            WindComputeState::Init => {
                let Some(init_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.init_pipeline) else {
                    return Ok(());
                };
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups((PARTICLE_COUNT + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1);
            }
            WindComputeState::Update => {
                let Some(update_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.update_pipeline) else {
                    return Ok(());
                };
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups((PARTICLE_COUNT + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE, 1, 1);
            }
        }

        Ok(())
    }
}
