use bevy::ecs::prelude::*;
use bevy::pbr2::StandardMaterial;
use bevy::prelude::{App, Assets, Handle, Plugin};
use bevy::render2::camera::Camera;
use bevy::render2::render_resource::Extent3d;
use bevy::render2::texture::Image;
use bevy::window::Windows;

use crate::render_to_texture::RenderToTexture;

pub struct CamDisplayPlugin;

impl Plugin for CamDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(swap_texture.label(CamDisplaySystem::SwapTextures))
            .add_system(
                resize_material_texture
                    .label(CamDisplaySystem::ResizeMaterialTexture)
                    .before(CamDisplaySystem::SwapTextures),
            );
    }
}

#[derive(SystemLabel, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CamDisplaySystem {
    SwapTextures,
    ResizeMaterialTexture,
}

pub struct CamDisplay {
    pub corresponding_camera: Entity,
}

fn swap_texture(
    cam_displays: Query<(&CamDisplay, &Handle<StandardMaterial>)>,
    mut cameras: Query<&mut RenderToTexture, With<Camera>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    for (cam_display, display_material) in cam_displays.iter() {
        let display_material = standard_materials.get_mut(display_material).unwrap();
        let material_texture = display_material.base_color_texture.as_mut().unwrap();

        let render_texture = &mut cameras.get_mut(cam_display.corresponding_camera).unwrap().0;

        std::mem::swap(material_texture, render_texture);
    }
}

fn resize_material_texture(
    cam_displays: Query<(&CamDisplay, &Handle<StandardMaterial>)>,
    cameras: Query<&Camera, With<RenderToTexture>>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    windows: Res<Windows>,
) {
    for (cam_display, material) in cam_displays.iter() {
        let material = standard_materials.get(material).unwrap();

        let camera = cameras.get(cam_display.corresponding_camera).unwrap();
        let window = windows.get(camera.window).unwrap();

        let new_size = Extent3d {
            width: window.physical_width(),
            height: window.physical_height(),
            depth_or_array_layers: 1,
        };

        let handle = material.base_color_texture.as_ref().unwrap();

        let texture = images.get(handle).unwrap();
        let texture_is_out_of_date = texture.texture_descriptor.size != new_size;

        if texture_is_out_of_date {
            let texture = images.get_mut(handle).unwrap();
            texture.resize(new_size);
        }
    }
}
