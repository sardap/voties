use std::ops::Sub;

use bevy::prelude::*;

use crate::{sim_time::SimTime, world_stats};

#[derive(Debug, Component, Clone, Copy)]
pub struct Energy {
    pub current_kcal: f64,
    pub max_kcal: f64,
    pub filled_percent: world_stats::Stat<f64>,
}

impl Energy {
    pub fn new(current_kcal: f64, max_kcal: f64) -> Self {
        Self {
            current_kcal,
            max_kcal,
            filled_percent: world_stats::Stat::default(),
        }
    }

    pub fn use_kcal(&mut self, kcal: f64) {
        self.current_kcal = self.current_kcal.sub(kcal);
        if self.current_kcal < 0.0 {
            self.current_kcal = 0.0;
        }
    }
}

pub fn drain_energy_system(time: Res<Time>, sim_time: Res<SimTime>, mut query: Query<&mut Energy>) {
    for mut energy in &mut query {
        energy.use_kcal(
            measurements::Volume::from_milliliters(100.0).as_milliliters()
                * sim_time.delta_seconds(&time) as f64,
        );
    }
}
