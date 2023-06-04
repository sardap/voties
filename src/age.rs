use bevy::prelude::*;

#[derive(Debug, Component, Default)]
pub struct Age {
    pub duration_alive: std::time::Duration,
}

pub fn age_up_system(time: Res<Time>, mut query: Query<&mut Age>) {
    for mut age in &mut query {
        age.duration_alive += time.delta();
    }
}
