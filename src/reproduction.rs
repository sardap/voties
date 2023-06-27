use std::time::Duration;

use bevy::prelude::*;

use crate::{collision, sim_time::SimTime};

#[derive(Debug, Component)]
pub struct Reproductive {
    pub reproduction_timer: Timer,
}

pub fn get_reproduction_cooldown(rng: &mut impl rand::Rng) -> Duration {
    pub const MIN_COOLDOWN: Duration = Duration::from_secs(30);
    pub const MAX_COOLDOWN: Duration = Duration::from_secs(60);

    rng.gen_range(MIN_COOLDOWN..MAX_COOLDOWN)
}

impl Reproductive {
    pub fn new(cool_down: Duration) -> Self {
        Self {
            reproduction_timer: Timer::new(cool_down, TimerMode::Repeating),
        }
    }

    pub fn wants_to_reproduce(&self) -> bool {
        false
    }
}

impl Default for Reproductive {
    fn default() -> Self {
        Self {
            reproduction_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
        }
    }
}

#[derive(Debug, Component)]
pub struct Pregnant {
    pub parents: [Entity; 2],
    pub pregnancy_timer: Timer,
}

pub fn get_pregnancy_duration(rng: &mut impl rand::Rng) -> Duration {
    const MIN_DURATION: Duration = Duration::from_secs(7);
    const MAX_DURATION: Duration = Duration::from_secs(10);

    rng.gen_range(MIN_DURATION..MAX_DURATION)
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

pub fn reproductive_timer_tick_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut q: Query<&mut Reproductive>,
) {
    for mut reproductive in &mut q {
        if !reproductive.reproduction_timer.finished() {
            reproductive.reproduction_timer.tick(sim_time.delta(&time));
        }
    }
}
