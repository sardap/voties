use std::time::Duration;

use bevy::prelude::*;

use crate::sim_time::SimTime;

#[derive(Debug, Default, Component)]
pub struct RequiresHouse {
    pub shelter: Option<Entity>,
    pub homeless_for: Duration,
}

pub fn tick_homeless_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut query: Query<&mut RequiresHouse>,
) {
    for mut requires_house in &mut query {
        if requires_house.shelter.is_some() {
            requires_house.homeless_for = Duration::ZERO;
            continue;
        }

        requires_house.homeless_for += sim_time.delta(&time);
    }
}
