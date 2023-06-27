use bevy::prelude::*;

use crate::{
    age::Age,
    death::{DeathReason, Mortal},
    name,
    sim_time::SimTime,
};

#[derive(Component, Clone)]
pub struct Grave {
    pub created: std::time::Duration,
    pub name: String,
    pub died_of: DeathReason,
    pub age: std::time::Duration,
}

pub fn create_grave_system(
    mut commands: Commands,
    sim_time: ResMut<SimTime>,
    query: Query<(&Mortal, &name::Name, &Age)>,
) {
    for (mortal, name, age) in &query {
        let death_reason = match mortal.dead {
            Some(reason) => reason,
            None => continue,
        };

        commands.spawn(Grave {
            created: sim_time.elapsed(),
            name: name.0.clone(),
            died_of: death_reason,
            age: age.duration_alive,
        });
    }
}
