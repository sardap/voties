use std::time::Duration;

use bevy::prelude::*;
use bevy_enum_filter::EnumFilter;

use crate::{
    farm::create_farm, hunger::FoodTemplate, mint, money_hole,
    reproduction::ReproductiveZoneBundle, upkeep,
};

#[derive(Debug, Resource)]
pub struct BuildingPlots {
    next_plot: Vec2,
}

const PLOT_START_X: f32 = -300.0;
const PLOT_START_Y: f32 = 200.0;
const PLOT_END_X: f32 = 300.0;
const PLOT_SIZE_X: f32 = 150.0;
const PLOT_SIZE_Y: f32 = 150.0;

impl BuildingPlots {
    pub fn new() -> Self {
        Self {
            next_plot: Vec2::new(PLOT_START_X, PLOT_START_Y),
        }
    }

    pub fn next(&mut self) -> Vec2 {
        let result = self.next_plot;

        self.next_plot += Vec2::new(PLOT_SIZE_X, 0.0);
        if self.next_plot.x > PLOT_END_X {
            self.next_plot.x = PLOT_START_X;
            self.next_plot.y -= PLOT_SIZE_Y;
        }

        result
    }
}

#[derive(Debug, Clone)]
pub enum Building {
    Farm(FoodTemplate),
    ReproductiveZone,
    MoneyHole,
    Mint,
}

impl ToString for Building {
    fn to_string(&self) -> String {
        match self {
            Building::Farm(food_template) => format!("Farm {}", food_template.name),
            Building::ReproductiveZone => "Reproductive Zone".to_string(),
            Building::MoneyHole => "Money Hole".to_string(),
            Building::Mint => "Mint".to_string(),
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
                    rng.gen_range(Duration::from_secs(2)..Duration::from_secs(4));
                mint::spawn(commands, asset_server, mps, production_cycle, location);
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
