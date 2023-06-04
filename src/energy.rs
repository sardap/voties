use std::ops::Sub;

use bevy::prelude::*;

#[derive(Debug, Component, Clone, Copy)]
pub struct Energy {
    pub current_kcal: f64,
    pub max_kcal: f64,
}

impl Energy {
    pub fn use_kcal(&mut self, kcal: f64) {
        self.current_kcal = self.current_kcal.sub(kcal);
        if self.current_kcal < 0.0 {
            self.current_kcal = 0.0;
        }
    }
}

pub fn drain_energy_system(time: Res<Time>, mut query: Query<&mut Energy>) {
    for mut energy in &mut query {
        energy.use_kcal(
            measurements::Volume::from_milliliters(100.0).as_milliliters()
                * time.delta_seconds() as f64,
        );
    }
}
