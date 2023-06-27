use bevy::prelude::*;

use crate::{
    money::{Money, Treasury},
    sim_time::SimTime,
};

#[derive(Debug, Clone, Component, Default)]
pub struct UpkeepCost {
    pub cost_per_second: Money,
    pub upkeep_lapsed: bool,
}

impl UpkeepCost {
    pub fn new(cost_per_second: Money) -> Self {
        Self {
            cost_per_second,
            upkeep_lapsed: false,
        }
    }
}

#[derive(Debug, Resource)]
pub struct UpkeepCostTimer(pub Timer);

pub fn upkeep_cost_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut upkeep_timer: ResMut<UpkeepCostTimer>,
    mut treasury: ResMut<Treasury>,
    mut query: Query<&mut UpkeepCost>,
) {
    let finished_count = upkeep_timer
        .0
        .tick(sim_time.delta(&time))
        .times_finished_this_tick();

    if finished_count == 0 {
        return;
    }

    for mut upkeep_cost in &mut query {
        let amount = upkeep_cost.cost_per_second * finished_count as f64;

        let updated_value = !treasury.spend(amount);

        if updated_value != upkeep_cost.upkeep_lapsed {
            upkeep_cost.upkeep_lapsed = updated_value;
        }
    }
}

pub fn setup_upkeep(commands: &mut Commands) {
    commands.insert_resource(UpkeepCostTimer(Timer::from_seconds(
        1.0,
        TimerMode::Repeating,
    )));
}
