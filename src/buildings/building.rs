use std::time::Duration;

use bevy::prelude::*;
use bevy_enum_filter::EnumFilter;

use crate::{hunger::FoodTemplate, reproduction::ReproductiveZoneBundle, upkeep};

use super::voting_center::VotingCenterBundle;
use super::{house, money_hole};

use super::farm::create_farm;

use super::mint;

const PLOT_START_X: f32 = -300.0;
const PLOT_START_Y: f32 = 200.0;
const PLOT_END_X: f32 = 300.0;
const PLOT_SIZE_X: f32 = 150.0;
const PLOT_SIZE_Y: f32 = 150.0;

#[derive(Debug, Resource)]
pub struct BuildingPlots {
    next_plot: Vec2,
    current_steps_in_spiral: i32,
    spiral_level: i32,
}

/*

   Steps 1
   0 1 2 3 4 5 6
   1 X X X X X X
   2 X X X X X X
   3 X X 0 X X X
   4 X N X X X X
   5 X X X X X X
   6 X X X X X X

   Steps 2
   0 1 2 3 4 5 6
   1 X X X X X X
   2 X X X X X X
   3 X N 0 X X X
   4 X B X X X X
   5 X X X X X X
   6 X X X X X X

   Steps 3
   0 1 2 3 4 5 6
   1 X X X X X X
   2 X N X X X X
   3 X B 0 X X X
   4 X B X X X X
   5 X X X X X X
   6 X X X X X X

   Steps 4
   0 1 2 3 4 5 6
   1 X X X X X X
   2 X B N X X X
   3 X B 0 X X X
   4 X B X X X X
   5 X X X X X X
   6 X X X X X X
*/

impl BuildingPlots {
    pub fn new() -> Self {
        Self {
            next_plot: Vec2::new(0.0, 0.0), // starting at the center
            current_steps_in_spiral: 0,
            spiral_level: 0,
        }
    }

    fn steps_in_spiral(&self) -> i32 {
        self.spiral_level * 8
    }

    pub fn next(&mut self) -> Vec2 {
        let result = self.next_plot;
        let steps_in_spiral = self.steps_in_spiral();

        if self.current_steps_in_spiral >= self.steps_in_spiral() {
            self.current_steps_in_spiral = 0;
            self.spiral_level += 1;
            self.next_plot -= Vec2::new(PLOT_SIZE_X, PLOT_SIZE_Y);
        } else {
            if self.current_steps_in_spiral < steps_in_spiral / 4 {
                self.next_plot += Vec2::new(0.0, PLOT_SIZE_Y);
            } else if self.current_steps_in_spiral < steps_in_spiral / 2 {
                self.next_plot += Vec2::new(PLOT_SIZE_X, 0.0);
            } else if self.current_steps_in_spiral < (steps_in_spiral * 3) / 4 {
                self.next_plot -= Vec2::new(0.0, PLOT_SIZE_Y);
            } else {
                self.next_plot -= Vec2::new(PLOT_SIZE_X, 0.0);
            }
            self.current_steps_in_spiral += 1;
        }

        result
    }
}
#[derive(Debug, Clone)]
pub enum Building {
    VotingCenter,
    Farm(FoodTemplate),
    ReproductiveZone,
    MoneyHole,
    Mint,
    House(i32),
}

impl ToString for Building {
    fn to_string(&self) -> String {
        match self {
            Building::VotingCenter => "Voting Center".to_owned(),
            Building::Farm(food_template) => format!("Farm {}", food_template.name),
            Building::ReproductiveZone => "Reproductive Zone".to_owned(),
            Building::MoneyHole => "Money Hole".to_owned(),
            Building::Mint => "Mint".to_owned(),
            Building::House(_) => "House".to_owned(),
        }
    }
}

impl Building {
    pub fn build(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        plots: &mut BuildingPlots,
        rng: &mut impl rand::Rng,
    ) {
        let location = plots.next();

        match self {
            Building::VotingCenter => {
                commands.spawn(VotingCenterBundle::new(asset_server, location));
            }
            Building::Farm(produces) => {
                create_farm(commands, asset_server, produces.clone(), location, rng);
            }
            Building::ReproductiveZone => {
                commands.spawn(ReproductiveZoneBundle::new(asset_server, location));
            }
            Building::MoneyHole => {
                let storage_capacity = rng.gen_range(
                    money_hole::MONEY_HOLE_CAPACITY_MIN..money_hole::MONEY_HOLE_CAPACITY_MAX,
                );
                money_hole::spawn(commands, asset_server, storage_capacity, location);
            }
            Building::Mint => {
                let mps = rng.gen_range(mint::MINT_MPS_MIN..mint::MINT_MPS_MAX);
                let production_cycle =
                    rng.gen_range(Duration::from_secs(1)..Duration::from_secs(2));
                mint::spawn(commands, asset_server, mps, production_cycle, location);
            }
            Building::House(dwellings) => {
                house::spawn(commands, asset_server, *dwellings, location);
            }
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Hash,
    enum_iterator::Sequence,
    Component,
    EnumFilter,
)]
pub enum BuildingStatus {
    Operational,
    Dilapidated,
}

impl Default for BuildingStatus {
    fn default() -> Self {
        Self::Operational
    }
}

impl num_traits::ToPrimitive for BuildingStatus {
    fn to_i64(&self) -> Option<i64> {
        Some(self.to_u64().unwrap() as i64)
    }

    fn to_u64(&self) -> Option<u64> {
        Some(match self {
            BuildingStatus::Operational => 0,
            BuildingStatus::Dilapidated => 1,
        })
    }
}

pub fn change_building_status_system(
    mut query: Query<(&mut BuildingStatus, Option<&upkeep::UpkeepCost>)>,
) {
    for (mut status, upkeep) in &mut query {
        let upkeep_met = if let Some(upkeep) = upkeep {
            !upkeep.upkeep_lapsed
        } else {
            true
        };

        let updated_status = if upkeep_met {
            BuildingStatus::Operational
        } else {
            BuildingStatus::Dilapidated
        };

        if updated_status != *status {
            *status = updated_status;
        }
    }
}

pub fn update_building_tint_system(
    mut query: Query<(&mut Sprite, &BuildingStatus), Changed<BuildingStatus>>,
) {
    for (mut farm, status) in query.iter_mut() {
        match status {
            BuildingStatus::Operational => {
                farm.color = Color::WHITE;
            }
            BuildingStatus::Dilapidated => {
                farm.color = Color::RED;
            }
        }
    }
}
