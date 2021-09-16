mod cam_display;
mod render_to_texture;

use std::f32::consts::TAU;

use bevy::core::Name;
use bevy::ecs::prelude::*;
use bevy::math::prelude::*;
use bevy::pbr2::{NotShadowCaster, PbrBundle, PointLight, PointLightBundle, StandardMaterial};
use bevy::prelude::{App, Assets, Transform};
use bevy::render2::camera::{ActiveCameras, PerspectiveCameraBundle};
use bevy::render2::color::Color;
use bevy::render2::mesh::{shape, Mesh};
use bevy::render2::texture::{BevyDefault, Image};
use bevy::PipelinedDefaultPlugins;
use bevy_inspector_egui::WorldInspectorPlugin;

use cam_display::{CamDisplay, CamDisplayPlugin};
use render_to_texture::{RenderToTexture, RenderToTexturePlugin};

mod utils;

fn main() {
    let mut app = App::new();
    app.add_plugins(PipelinedDefaultPlugins)
        .add_plugin(RenderToTexturePlugin)
        .add_plugin(CamDisplayPlugin)
        .add_plugin(utils::FlycamPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup.system())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut active_cameras: ResMut<ActiveCameras>,
    mut images: ResMut<Assets<Image>>,
) {
    let cam_1_render_texture = images.add(dummy_image());
    let cam_1_material_texture = images.add(dummy_image());

    let cam_2_render_texture = images.add(dummy_image());
    let cam_2_material_texture = images.add(dummy_image());

    // Regular camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0), //.looking_at(Vec3::default(), Vec3::Y),
            ..Default::default()
        })
        .insert(utils::Flycam)
        .insert(Name::new("regular camera"));

    // Additional cameras
    let additional_cam_1 = commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, -5.0).looking_at(Vec3::default(), Vec3::Y),
            ..PerspectiveCameraBundle::with_name("additional camera 1")
        })
        .insert(RenderToTexture(cam_1_render_texture))
        .insert(Name::new("camera 1"))
        .id();
    active_cameras.add("additional camera 1");

    let additional_cam_2 = commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0)
                .looking_at(Vec3::new(-2.0, 2.5, -5.0), Vec3::Y),
            ..PerspectiveCameraBundle::with_name("additional camera 2")
        })
        .insert(RenderToTexture(cam_2_render_texture))
        .insert(Name::new("camera 2"))
        .id();
    active_cameras.add("additional camera 2");

    // Environment
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert(Name::new("Floor"));
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Cube"));
    commands
        .spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 3100.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        })
        .insert(Name::new("Point light"));

    // Camera display planes
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb_u8(242, 240, 240),
                base_color_texture: Some(cam_1_material_texture),
                unlit: true,
                ..Default::default()
            }),
            transform: Transform {
                translation: Vec3::new(-1.3, 1.5, -1.0),
                rotation: Quat::from_euler(bevy::math::EulerRot::XYZ, TAU / 4.0, TAU / 2.0, 0.0),
                scale: Vec3::new(1.77, 1.0, 1.0),
            },
            ..Default::default()
        })
        .insert(NotShadowCaster)
        .insert(Name::new("Plane 1"))
        .insert(CamDisplay {
            corresponding_camera: additional_cam_1,
        });

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb_u8(242, 240, 240),
                base_color_texture: Some(cam_2_material_texture),
                unlit: true,
                ..Default::default()
            }),
            transform: Transform {
                translation: Vec3::new(1.5, 1.5, -0.2),
                rotation: Quat::from_euler(bevy::math::EulerRot::XYZ, TAU / 4.0, TAU / 2.0, -0.5),
                scale: Vec3::new(1.77, 1.0, 1.0),
            },
            ..Default::default()
        })
        .insert(NotShadowCaster)
        .insert(Name::new("Plane 2"))
        .insert(CamDisplay {
            corresponding_camera: additional_cam_2,
        });
}

fn dummy_image() -> Image {
    use bevy::render2::render_resource::*;

    Image {
        data: vec![1; 4],
        texture_descriptor: TextureDescriptor {
            label: None,
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            format: TextureFormat::bevy_default(),
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_DST,
        },
        sampler_descriptor: SamplerDescriptor::default(),
    }
}
