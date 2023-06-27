use bevy::prelude::*;

use crate::collision;

#[derive(Component, Clone, Default)]
pub struct VotingCenter;

#[derive(Bundle, Clone, Default)]
pub struct VotingCenterBundle {
    pub voting_center: VotingCenter,
    pub collider: collision::Collider,
    #[bundle]
    pub sprite: SpriteBundle,
}

impl VotingCenterBundle {
    pub fn new(asset_server: &AssetServer, location: Vec2) -> Self {
        Self {
            voting_center: VotingCenter,
            collider: collision::Collider::default(),
            sprite: SpriteBundle {
                texture: asset_server.load(crate::assets::DEFAULT_VOTING_CENTER),
                transform: Transform::from_translation(Vec3::new(location.x, location.y, 0.0)),
                ..default()
            },
        }
    }
}
