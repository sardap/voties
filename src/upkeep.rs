use bevy::prelude::*;

use crate::money::{Money, Treasury};

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
    mut upkeep_timer: ResMut<UpkeepCostTimer>,
    mut treasury: ResMut<Treasury>,
    mut query: Query<&mut UpkeepCost>,
) {
    let elapsed = upkeep_timer.0.elapsed_secs() as f64 + time.delta_seconds_f64();

    if !upkeep_timer.0.tick(time.delta()).just_finished() {
        return;
    }

    for mut upkeep_cost in &mut query {
        let amount = upkeep_cost.cost_per_second * elapsed;

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
