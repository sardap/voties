use std::time::Duration;

use bevy::prelude::*;

use crate::collision;

#[derive(Debug, Component)]
pub struct Reproductive {
    pub next_reproduction: Duration,
}

impl Reproductive {
    pub fn wants_to_reproduce(&self) -> bool {
        false
    }
}

impl Default for Reproductive {
    fn default() -> Self {
        Self {
            next_reproduction: Duration::ZERO,
        }
    }
}

#[derive(Debug, Component)]
pub struct Pregnant {
    pub parents: [Entity; 2],
    pub pregnant_since: Duration,
    pub pregnancy_duration: Duration,
}

#[derive(Debug, Component, Clone)]
pub struct ReproductiveZone {
    pub max_capacity: usize,
}

impl Default for ReproductiveZone {
    fn default() -> Self {
        Self { max_capacity: 2 }
    }
}

#[derive(Bundle, Clone, Default)]
pub struct ReproductiveZoneBundle {
    pub reproductive: ReproductiveZone,
    pub collider: collision::Collider,
    pub collision_holder: collision::CollisionHolder,
    #[bundle]
    pub sprite: SpriteBundle,
}

impl ReproductiveZoneBundle {
    pub fn new(asset_server: &AssetServer, transform: Vec2) -> Self {
        Self {
            sprite: SpriteBundle {
                texture: asset_server.load(crate::assets::DEFAULT_REPRODUCTIVE_ZONE),
                transform: Transform::from_xyz(transform.x, transform.y, 0.0),
                ..default()
            },
            ..default()
        }
    }
}
