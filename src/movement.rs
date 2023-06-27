use bevy::prelude::*;

use crate::sim_time::SimTime;

#[derive(Component, Default)]
pub struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug, Component)]
pub struct MovementSpeed(pub f32);

#[derive(Debug, Component)]
pub struct MovementGoal {
    pub target: Option<Vec3>,
}

impl Default for MovementGoal {
    fn default() -> Self {
        Self { target: None }
    }
}

pub fn go_to_target(mut query: Query<(&MovementGoal, &MovementSpeed, &Transform, &mut Velocity)>) {
    for (movement_goal, speed, transform, mut vel) in &mut query {
        if let Some(target) = movement_goal.target {
            let direction = (target - transform.translation).normalize();
            let velocity = direction * speed.0;
            vel.x = velocity.x;
            vel.y = velocity.y;
        }
    }
}

pub fn apply_velcoity_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    for (mut transform, vel) in &mut query {
        transform.translation.x += (vel.x * sim_time.delta_seconds(&time)).min(50.0);
        transform.translation.y += (vel.y * sim_time.delta_seconds(&time)).min(50.0);
    }
}
