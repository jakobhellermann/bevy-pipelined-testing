mod render_to_texture;

use std::f32::consts::TAU;

use bevy::core::Name;
use bevy::ecs::prelude::*;
use bevy::math::prelude::*;
use bevy::pbr2::{NotShadowCaster, PbrBundle, PointLight, PointLightBundle, StandardMaterial};
use bevy::prelude::{App, Assets, Handle, Transform};
use bevy::render2::camera::{ActiveCameras, Camera, PerspectiveCameraBundle};
use bevy::render2::color::Color;
use bevy::render2::mesh::{shape, Mesh};
use bevy::render2::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsage};
use bevy::render2::texture::{BevyDefault, Image};
use bevy::PipelinedDefaultPlugins;
use bevy_inspector_egui::WorldInspectorPlugin;

use render_to_texture::{camera, RenderToTexturePlugin};

use crate::render_to_texture::RenderToTexture;

fn main() {
    let mut app = App::new();
    // app.add_plugins_with(PipelinedDefaultPlugins, |p| p.disable::<bevy::log::LogPlugin>())
    app.add_plugins(PipelinedDefaultPlugins)
        .add_plugin(RenderToTexturePlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_startup_system(setup.system())
        .add_system(swap_texture)
        .run();
}

struct CamDisplay {
    corresponding_camera: Entity,
}

fn swap_texture(
    cam_display: Query<(&CamDisplay, &Handle<StandardMaterial>)>,
    mut cameras: Query<&mut RenderToTexture, With<Camera>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    for (cam_display, display_material) in cam_display.iter() {
        let display_material = standard_materials.get_mut(display_material).unwrap();
        let material_texture = display_material.base_color_texture.as_mut().unwrap();

        let render_texture = &mut cameras.get_mut(cam_display.corresponding_camera).unwrap().0;

        std::mem::swap(material_texture, render_texture);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut active_cameras: ResMut<ActiveCameras>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut img = Image::new(
        Extent3d {
            width: 1280,
            height: 720,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![1; 4 * 1280 * 720],
        TextureFormat::bevy_default(),
    );
    img.texture_descriptor.usage =
        TextureUsage::RENDER_ATTACHMENT | TextureUsage::SAMPLED | TextureUsage::COPY_DST;
    let cam_2_texture_render = images.add(img.clone());
    let cam_2_texture_material = images.add(img);

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });

    let rtt_cam = commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, -5.0).looking_at(Vec3::default(), Vec3::Y),
            ..PerspectiveCameraBundle::with_name(camera::CAM_2)
        })
        .insert(RenderToTexture(cam_2_texture_render))
        .id();
    active_cameras.add(camera::CAM_2);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(Name::new("Cube"));

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(cam_2_texture_material),
                unlit: true,
                ..Default::default()
            }),
            transform: Transform {
                translation: Vec3::new(-1.3, 1.5, -1.0),
                rotation: Quat::from_euler(bevy::math::EulerRot::XYZ, TAU / 4.0, TAU / 2.0, 0.0),
                scale: Vec3::ONE,
            },
            ..Default::default()
        })
        .insert(NotShadowCaster)
        .insert(Name::new("Plane"))
        .insert(CamDisplay {
            corresponding_camera: rtt_cam,
        });

    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 3100.0,
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
}
