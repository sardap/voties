use bevy::prelude::*;

use crate::sim_time::SimTime;

#[derive(Debug, Component, Default)]
pub struct Age {
    pub duration_alive: std::time::Duration,
}

pub fn age_up_system(time: Res<Time>, sim_time: Res<SimTime>, mut query: Query<&mut Age>) {
    for mut age in &mut query {
        age.duration_alive += sim_time.delta(&time);
    }
}
