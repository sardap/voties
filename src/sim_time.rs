use std::time::Duration;

use bevy::prelude::*;

#[derive(Debug, Resource)]
pub struct SimTime {
    multiplier: f32,
    current_time: Duration,
}

impl Default for SimTime {
    fn default() -> Self {
        Self {
            multiplier: 1.0,
            current_time: Duration::ZERO,
        }
    }
}

impl SimTime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_to_multiplier(&mut self, to_add: f32) {
        self.multiplier += to_add;
        self.multiplier = self.multiplier.clamp(0.1, 20.0);
    }

    pub fn delta(&self, time: &Time) -> Duration {
        time.delta().mul_f32(self.multiplier)
    }

    pub fn delta_seconds(&self, time: &Time) -> f32 {
        time.delta_seconds() * self.multiplier
    }

    pub fn delta_seconds_f64(&self, time: &Time) -> f64 {
        time.delta_seconds_f64() * self.multiplier as f64
    }

    pub fn elapsed(&self) -> Duration {
        self.current_time
    }
}

pub fn setup(commands: &mut Commands) {
    commands.insert_resource(SimTime::new());
}

pub fn tick_sim_time_system(time: Res<Time>, mut sim_time: ResMut<SimTime>) {
    let delta = sim_time.delta(&time);
    sim_time.current_time += delta;
}
