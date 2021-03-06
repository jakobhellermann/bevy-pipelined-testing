use bevy::core_pipeline::node::MAIN_PASS_DEPENDENCIES;
use bevy::core_pipeline::{draw_3d_graph, Transparent3d, ViewDepthTexture};
use bevy::ecs::prelude::*;
use bevy::prelude::{App, Assets, Handle, Plugin};
use bevy::render2::camera::Camera;
use bevy::render2::render_asset::RenderAssets;
use bevy::render2::render_graph::{self, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render2::render_phase::RenderPhase;
use bevy::render2::render_resource::Extent3d;
use bevy::render2::renderer::RenderContext;
use bevy::render2::texture::Image;
use bevy::render2::{RenderApp, RenderStage};
use bevy::window::Windows;

pub mod node {
    pub const RENDER_TO_TEXTURE: &str = "render_to_texture_node";
}

pub struct RenderToTexture(pub Handle<Image>);

pub struct RenderToTexturePlugin;
impl Plugin for RenderToTexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(resize_rtt_texture.label(RenderToTextureSystem::ResizeTexture));

        let render_app = app.sub_app(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract_rtt_render_phase);

        let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
        render_graph.add_node(node::RENDER_TO_TEXTURE, SecondCamDriverNode::new());
        render_graph
            .add_node_edge(node::RENDER_TO_TEXTURE, MAIN_PASS_DEPENDENCIES)
            .unwrap();
    }
}

#[derive(SystemLabel, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RenderToTextureSystem {
    ResizeTexture,
}

fn resize_rtt_texture(
    cams: Query<(&RenderToTexture, &Camera)>,
    mut images: ResMut<Assets<Image>>,
    windows: Res<Windows>,
) {
    for (render_to_texture, camera) in cams.iter() {
        let window = windows.get(camera.window).unwrap();

        let new_size = Extent3d {
            width: window.physical_width(),
            height: window.physical_height(),
            depth_or_array_layers: 1,
        };

        let texture = images.get(&render_to_texture.0).unwrap();
        let texture_is_out_of_date = texture.texture_descriptor.size != new_size;

        if texture_is_out_of_date {
            let texture = images.get_mut(&render_to_texture.0).unwrap();
            texture.resize(new_size);
        }
    }
}

fn extract_rtt_render_phase(
    mut commands: Commands,
    cams: Query<(Entity, &RenderToTexture), With<Camera>>,
) {
    for (entity, render_to_texture) in cams.iter() {
        let mut entity = commands.get_or_spawn(entity);

        entity.insert(RenderPhase::<Transparent3d>::default());
        entity.insert(RenderToTexture(render_to_texture.0.clone_weak()));
    }
}

struct SecondCamDriverNode {
    query: Option<QueryState<Entity, With<RenderToTexture>>>,
    rtt_cameras: Vec<Entity>,
}
impl SecondCamDriverNode {
    fn new() -> SecondCamDriverNode {
        SecondCamDriverNode {
            query: None,
            rtt_cameras: Vec::new(),
        }
    }
}

impl render_graph::Node for SecondCamDriverNode {
    fn update(&mut self, world: &mut World) {
        let query_state = self.query.get_or_insert_with(|| QueryState::new(world));
        query_state.update_archetypes(world);

        self.rtt_cameras = query_state.iter(world).collect();
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        for &camera_entity in &self.rtt_cameras {
            let render_to_texture = world.get::<RenderToTexture>(camera_entity).unwrap();
            let depth_texture = world.get::<ViewDepthTexture>(camera_entity).unwrap();

            let image_render_assets = world.get_resource::<RenderAssets<Image>>().unwrap();
            let gpu_image = &image_render_assets[&render_to_texture.0];

            graph.run_sub_graph(
                draw_3d_graph::NAME,
                vec![
                    SlotValue::Entity(camera_entity),
                    SlotValue::TextureView(gpu_image.texture_view.clone()),
                    SlotValue::TextureView(depth_texture.view.clone()),
                ],
            )?;
        }

        Ok(())
    }
}
