use bevy::ecs::prelude::*;
use bevy::math::Vec3;
use bevy::prelude::{App, AssetServer, Assets, Transform};
use bevy::render2::camera::PerspectiveCameraBundle;
use bevy::render2::mesh::{shape, Mesh};
use bevy::PipelinedDefaultPlugins;

use bevy_portals::screenspace_texture::{
    ScreenspaceTextureBundle, ScreenspaceTextureMaterial, ScreenspaceTexturePlugin,
};
use bevy_portals::utils::Flycam;

fn main() {
    App::new()
        .add_plugins(PipelinedDefaultPlugins)
        .add_plugin(ScreenspaceTexturePlugin)
        .add_plugin(bevy_portals::utils::FlycamPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ScreenspaceTextureMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load("texture.png");

    commands.spawn().insert_bundle(ScreenspaceTextureBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(ScreenspaceTextureMaterial { texture }),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..Default::default()
    });

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(Flycam);
}
