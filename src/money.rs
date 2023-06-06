use bevy::prelude::*;

pub type Money = f64;

#[derive(Debug, Resource)]
pub struct Treasury {
    pub money: Money,
}

impl Treasury {
    pub fn new() -> Self {
        Self { money: 0.0 }
    }

    pub fn spend(&mut self, amount: Money) -> bool {
        if self.money >= amount {
            self.money -= amount;
            true
        } else {
            false
        }
    }

    pub fn add(&mut self, amount: Money) {
        self.money += amount;
    }
}

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

pub fn upkeep_cost_system(
    time: Res<Time>,
    mut treasury: ResMut<Treasury>,
    mut query: Query<&mut UpkeepCost>,
) {
    for mut upkeep_cost in &mut query {
        let amount = upkeep_cost.cost_per_second * time.delta_seconds_f64();

        let updated_value = !treasury.spend(amount);

        if updated_value != upkeep_cost.upkeep_lapsed {
            upkeep_cost.upkeep_lapsed = updated_value;
        }
    }
}
