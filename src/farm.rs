use std::time::Duration;

use bevy::prelude::*;

use crate::{
    assets, collision,
    hunger::{self, FoodTemplate},
};

#[derive(Component, Clone, Default)]
pub struct Farm {
    pub surplus: i32,
    pub production_time: Duration,
    pub time_spent_producing: Duration,
}

impl Farm {
    pub fn take_food(&mut self) -> bool {
        if self.surplus <= 0 {
            return false;
        }
        self.surplus -= 1;
        true
    }

    pub fn has_surplus(&self) -> bool {
        self.surplus > 0
    }
}

#[derive(Bundle, Clone, Default)]
pub struct FarmBundle {
    pub farm: Farm,
    pub produces: hunger::Food,
    pub collider: collision::Collider,
    #[bundle]
    pub sprite: SpriteBundle,
}

#[derive(Component, Clone, Default)]
pub struct FarmText;

#[derive(Bundle, Clone, Default)]
pub struct FarmTextBundle {
    pub farm_text: FarmText,
    #[bundle]
    pub text2d: Text2dBundle,
}

struct ProductionRange {
    min: Duration,
    max: Duration,
}

impl ProductionRange {
    fn from_secs(min: u64, max: u64) -> Self {
        Self {
            min: Duration::from_secs(min),
            max: Duration::from_secs(max),
        }
    }

    fn from_millis(min: u64, max: u64) -> Self {
        Self {
            min: Duration::from_millis(min),
            max: Duration::from_millis(max),
        }
    }

    fn new(min: Duration, max: Duration) -> Self {
        Self { min, max }
    }
}

fn get_production_time_range(difficulty: u64) -> ProductionRange {
    let mut min = Duration::from_secs(1);
    min += Duration::from_millis((difficulty * 1000).checked_div(4).unwrap_or(0));

    let mut max = Duration::from_secs(5);
    max += Duration::from_millis((difficulty * 1000).checked_div(2).unwrap_or(0));

    ProductionRange::new(min, max)
}

pub fn create_farm(
    commands: &mut Commands,
    asset_server: &AssetServer,
    produces: FoodTemplate,
    location: Vec3,
    rng: &mut impl rand::Rng,
) {
    let production_range = get_production_time_range(produces.difficulty as u64);
    let production_time = rng.gen_range(production_range.min..production_range.max);
    let farm_name = format!(
        "{} Farm - {} seconds",
        produces.name,
        production_time.as_secs()
    );

    let farm_id = commands
        .spawn(FarmBundle {
            farm: Farm {
                production_time,
                surplus: 0,
                time_spent_producing: Duration::ZERO,
            },
            produces: produces.into(),
            sprite: SpriteBundle {
                texture: asset_server.load(crate::assets::DEFAULT_FARM_SPRITE_PATH),
                transform: Transform::from_translation(location),
                ..default()
            },
            ..default()
        })
        .id();

    let text_id = commands
        .spawn(FarmTextBundle {
            farm_text: FarmText,
            text2d: Text2dBundle {
                text: Text::from_section(
                    farm_name,
                    TextStyle {
                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                        font_size: 10.0,
                        color: Color::BLACK,
                    },
                ),
                transform: Transform::from_xyz(0.0, 60.0, 10.0),
                ..default()
            },
        })
        .id();

    commands.entity(farm_id).add_child(text_id);
}

pub fn farms_make_food_system(time: Res<Time>, mut query: Query<&mut Farm>) {
    for mut farm in query.iter_mut() {
        let mut time_spent_producing = farm.time_spent_producing;
        time_spent_producing += time.delta();
        while time_spent_producing >= farm.production_time {
            time_spent_producing -= farm.production_time;
            farm.surplus += 1;
        }
        farm.time_spent_producing = time_spent_producing;
    }
}
