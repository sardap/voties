use bevy::prelude::*;

use crate::{sim_time::SimTime, ui::camera::GameCamera};

pub fn player_input_camera_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<(&mut Transform, &mut OrthographicProjection), With<GameCamera>>,
) {
    {
        let (_, mut projection) = camera.single_mut();

        if keyboard_input.just_pressed(KeyCode::Equals) {
            projection.scale *= 0.75;
        }

        if keyboard_input.just_pressed(KeyCode::Minus) {
            projection.scale *= 1.25;
        }

        projection.scale = projection.scale.clamp(0.25, 10.0);
    }

    {
        let (mut transform, _) = camera.single_mut();

        if keyboard_input.pressed(KeyCode::Left) {
            transform.translation.x -= 10.0;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            transform.translation.x += 10.0;
        }

        if keyboard_input.pressed(KeyCode::Up) {
            transform.translation.y += 10.0;
        }

        if keyboard_input.pressed(KeyCode::Down) {
            transform.translation.y -= 10.0;
        }
    }
}

pub const SIM_TIME_MULTIPLIER_STEP: f32 = 0.5;

pub fn player_input_sim_time_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut sim_time: ResMut<SimTime>,
) {
    if keyboard_input.just_pressed(KeyCode::O) {
        sim_time.add_to_multiplier(-SIM_TIME_MULTIPLIER_STEP);
    }

    if keyboard_input.just_pressed(KeyCode::P) {
        sim_time.add_to_multiplier(SIM_TIME_MULTIPLIER_STEP);
    }
}
