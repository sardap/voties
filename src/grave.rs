use bevy::prelude::*;

use crate::assets;

#[derive(Component, Clone)]
pub struct Grave {
    pub created: std::time::Duration,
    pub name: String,
    pub died_of: String,
    pub age: std::time::Duration,
}

#[derive(Bundle, Clone)]
pub struct GraveBundle {
    grave: Grave,
    #[bundle]
    sprite: SpriteBundle,
}

impl GraveBundle {
    pub fn new(
        asset_server: &Res<AssetServer>,
        source_position: &Transform,
        time: &Res<Time>,
        name: &str,
        died_of: &str,
        age: std::time::Duration,
    ) -> Self {
        Self {
            grave: Grave {
                created: time.elapsed(),
                name: name.to_string(),
                died_of: died_of.to_string(),
                age,
            },
            sprite: SpriteBundle {
                texture: asset_server.load(assets::DEFAULT_GRAVE_SPRITE_PATH),
                transform: source_position.clone(),
                ..default()
            },
        }
    }
}
