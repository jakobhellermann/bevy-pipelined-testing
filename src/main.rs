use std::f32::consts::TAU;

use bevy::core::Name;
use bevy::core_pipeline::node::MAIN_PASS_DEPENDENCIES;
use bevy::core_pipeline::{draw_3d_graph, Transparent3dPhase, ViewDepthTexture};
use bevy::ecs::prelude::*;
use bevy::math::prelude::*;
use bevy::pbr2::{NotShadowCaster, PbrBundle, PointLight, PointLightBundle, StandardMaterial};
use bevy::prelude::{App, Assets, Handle, Transform};
use bevy::render2::camera::{ActiveCameras, Camera, ExtractedCameraNames, PerspectiveCameraBundle};
use bevy::render2::color::Color;
use bevy::render2::mesh::{shape, Mesh};
use bevy::render2::render_asset::RenderAssets;
use bevy::render2::render_graph::{self, RenderGraph, SlotValue};
use bevy::render2::render_phase::RenderPhase;
use bevy::render2::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsage};
use bevy::render2::texture::{BevyDefault, Image};
use bevy::render2::{RenderApp, RenderStage};
use bevy::PipelinedDefaultPlugins;
use bevy_inspector_egui::WorldInspectorPlugin;

const CAM_2: &str = "cam_2";

fn main() {
    let mut app = App::new();
    // app.add_plugins_with(PipelinedDefaultPlugins, |p| p.disable::<bevy::log::LogPlugin>())
    app.add_plugins(PipelinedDefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup.system());

    let render_app = app.sub_app(RenderApp);
    render_app.add_system_to_stage(RenderStage::Extract, extract_cam2_render_phase);

    let mut render_graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();
    render_graph.add_node("second_cam_node", SecondCamDriverNode);
    render_graph
        .add_node_edge("second_cam_node", MAIN_PASS_DEPENDENCIES)
        .unwrap();

    // bevy_mod_debugdump::print_render_graph(&mut app);

    app.run();
}

struct SecondCamDriverNode;

impl render_graph::Node for SecondCamDriverNode {
    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        _render_context: &mut bevy::render2::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let extracted_cameras = world.get_resource::<ExtractedCameraNames>().unwrap();
        let camera_entity = *extracted_cameras.entities.get(CAM_2).unwrap();

        let cam2 = world.get::<Cam2>(camera_entity).unwrap();
        let depth_texture = world.get::<ViewDepthTexture>(camera_entity).unwrap();

        let image_render_assets = world.get_resource::<RenderAssets<Image>>().unwrap();
        let gpu_image = &image_render_assets[&cam2.render_to_texture];

        graph.run_sub_graph(
            draw_3d_graph::NAME,
            vec![
                SlotValue::Entity(camera_entity),
                SlotValue::TextureView(gpu_image.texture_view.clone()),
                SlotValue::TextureView(depth_texture.view.clone()),
            ],
        )?;

        Ok(())
    }
}

#[derive(Clone)]
struct Cam2 {
    render_to_texture: Handle<Image>,
}

fn extract_cam2_render_phase(mut commands: Commands, cams: Query<(Entity, &Cam2), With<Camera>>) {
    for (entity, cam2) in cams.iter() {
        let mut entity = commands.get_or_spawn(entity);

        entity.insert(RenderPhase::<Transparent3dPhase>::default());
        entity.insert(cam2.clone());
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut active_cameras: ResMut<ActiveCameras>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    // let cam_2_texture = images.add(Image::default());
    let mut img = Image::new_fill(
        Extent3d {
            width: 1280,
            height: 720,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[1, 1, 1, 1],
        TextureFormat::bevy_default(),
    );
    img.texture_descriptor.usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_DST;
    let cam_2_texture = images.add(img);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(StandardMaterial {
                // base_color_texture: Some(cam_2_texture.clone()),
                ..Default::default()
            }),
            transform: Transform {
                translation: Vec3::new(-1.3, 1.5, -1.0),
                rotation: Quat::from_euler(bevy::math::EulerRot::XYZ, TAU / 4.0, 0.0, 0.0),
                scale: Vec3::ONE,
            },
            ..Default::default()
        })
        .insert(NotShadowCaster)
        .insert(Name::new("Plane"));

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 3100.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, -5.0).looking_at(Vec3::default(), Vec3::Y),
            ..PerspectiveCameraBundle::with_name(CAM_2)
        })
        .insert(Cam2 {
            render_to_texture: cam_2_texture,
        });
    active_cameras.add(CAM_2);
}
