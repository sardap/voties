use std::time::Duration;

use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    assets, collision,
    hunger::{self, FoodTemplate},
    sim_time::SimTime,
    upkeep,
};

use super::building;

#[derive(Component, Clone, Default)]
pub struct Farm {
    pub surplus: i32,
    pub production_timer: Timer,
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

struct ProductionRange {
    min: Duration,
    max: Duration,
}

impl ProductionRange {
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

#[derive(Bundle, Clone, Default)]
pub struct FarmBundle {
    pub farm: Farm,
    pub produces: hunger::Food,
    pub collider: collision::Collider,
    pub building_status: building::BuildingStatus,
    pub upkeep: upkeep::UpkeepCost,
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

pub fn create_farm(
    commands: &mut Commands,
    asset_server: &AssetServer,
    produces: FoodTemplate,
    location: Vec2,
    rng: &mut impl rand::Rng,
) {
    let upkeep_cost = produces.upkeep_cost();

    let production_range = get_production_time_range(produces.difficulty as u64);
    let production_time = rng.gen_range(production_range.min..production_range.max) / 2;
    let farm_name = format!(
        "{} Farm - {:.2}/s ${:.2}/s",
        produces.name,
        production_time.as_secs_f32(),
        upkeep_cost
    );

    let farm_id = commands
        .spawn(FarmBundle {
            upkeep: upkeep::UpkeepCost::new(upkeep_cost),
            farm: Farm {
                surplus: 0,
                production_timer: Timer::from_seconds(
                    production_time.as_secs_f32(),
                    TimerMode::Repeating,
                ),
            },
            produces: produces.into(),
            sprite: SpriteBundle {
                texture: asset_server.load(crate::assets::DEFAULT_FARM_SPRITE_PATH),
                transform: Transform::from_translation(Vec3::new(location.x, location.y, 0.0)),
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

pub fn farms_make_food_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut query: Query<&mut Farm, With<Enum!(building::BuildingStatus::Operational)>>,
) {
    for mut farm in query.iter_mut() {
        let new_surplus = farm
            .production_timer
            .tick(sim_time.delta(&time))
            .times_finished_this_tick();

        if new_surplus > 0 {
            // TODO make text float up
            farm.surplus += new_surplus as i32;
        }
    }
}
