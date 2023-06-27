use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct GameCamera;

pub fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            camera_2d: Camera2d {
                // clear_color: ClearColorConfig::Custom(Color::WHITE),
                ..default()
            },
            ..default()
        })
        .insert(GameCamera);
}
