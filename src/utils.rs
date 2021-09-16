use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::WindowFocused;

pub struct FlycamPlugin;
impl Plugin for FlycamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlycamOptions>()
            .add_system(camera_movement)
            .add_system(camera_look)
            .add_system(toggle_cursor)
            .add_system(toggle_cursor_manually);
    }
}

pub struct FlycamOptions {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
    pub enabled: bool,
}
impl Default for FlycamOptions {
    fn default() -> Self {
        Self {
            yaw: Default::default(),
            pitch: Default::default(),
            sensitivity: 3.0,
            enabled: true,
        }
    }
}

pub struct Flycam;

fn camera_movement(
    mut cam: Query<&mut Transform, With<Flycam>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,

    options: ResMut<FlycamOptions>,
) {
    if !options.enabled {
        return;
    }

    let mut camera = cam.single_mut();

    let if_then_1 = |b| if b { 1.0 } else { 0.0 };
    let forward = if_then_1(keyboard_input.pressed(KeyCode::W))
        - if_then_1(keyboard_input.pressed(KeyCode::S));
    let sideways = if_then_1(keyboard_input.pressed(KeyCode::D))
        - if_then_1(keyboard_input.pressed(KeyCode::A));
    let up = if_then_1(keyboard_input.pressed(KeyCode::Space))
        - if_then_1(keyboard_input.pressed(KeyCode::LControl));

    if forward == 0.0 && sideways == 0.0 && up == 0.0 {
        return;
    }

    let speed = if keyboard_input.pressed(KeyCode::LShift) {
        20.0
    } else {
        5.0
    };

    let movement =
        Vec3::new(sideways, forward, up).normalize_or_zero() * speed * time.delta_seconds();

    let diff =
        camera.forward() * movement.y + camera.right() * movement.x + camera.up() * movement.z;
    camera.translation += diff;
}

fn camera_look(
    time: Res<Time>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Flycam>>,
    mut options: ResMut<FlycamOptions>,
) {
    if !options.enabled {
        return;
    }
    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_event_reader.iter() {
        delta += event.delta;
    }
    if delta.is_nan() || delta.abs_diff_eq(Vec2::ZERO, f32::EPSILON) {
        return;
    }

    for mut transform in query.iter_mut() {
        options.yaw -= delta.x * options.sensitivity * time.delta_seconds();
        options.pitch += delta.y * options.sensitivity * time.delta_seconds();

        options.pitch = options.pitch.clamp(-89.0, 89.9);
        // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

        let yaw_radians = options.yaw.to_radians();
        let pitch_radians = options.pitch.to_radians();

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
            * Quat::from_axis_angle(-Vec3::X, pitch_radians);
    }
}

fn toggle_cursor(
    mut window_focused_events: EventReader<WindowFocused>,
    mut windows: ResMut<Windows>,
) {
    for &WindowFocused { id, focused } in window_focused_events.iter() {
        let window = windows.get_mut(id).unwrap();
        window.set_cursor_lock_mode(focused);
        window.set_cursor_visibility(!focused);
    }
}

fn toggle_cursor_manually(
    keyboard_events: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    mut flycam_options: ResMut<FlycamOptions>,
) {
    if keyboard_events.just_pressed(KeyCode::Escape) {
        flycam_options.enabled = !flycam_options.enabled;

        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(!window.cursor_locked());
        window.set_cursor_visibility(!window.cursor_visible());
    }
}
