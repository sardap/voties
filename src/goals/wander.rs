use bevy::prelude::*;
use bevy_enum_filter::prelude::*;
use rand::Rng;

use crate::{movement, rng};

use super::Goals;

#[derive(Debug, Clone, Copy)]
pub enum WanderState {
    FindingTarget,
    MovingToTarget,
}

impl Default for WanderState {
    fn default() -> Self {
        WanderState::FindingTarget
    }
}

pub fn step_wander_goal_system(
    mut rng: ResMut<rng::Rng>,
    mut query: Query<
        (&mut Goals, &mut movement::MovementGoal, &Transform),
        With<Enum!(super::Goals::Wander)>,
    >,
) {
    for (mut goal, mut movement_goal, position) in &mut query {
        let wander = match goal.clone() {
            Goals::Wander(x) => x,
            _ => todo!(),
        };

        match wander {
            WanderState::FindingTarget => {
                let new_target = position.translation
                    + Vec3::new(
                        rng.inner.gen_range(-100.0..100.0),
                        rng.inner.gen_range(-100.0..100.0),
                        0.0,
                    );
                movement_goal.target = Some(new_target);
                *goal = Goals::Wander(WanderState::MovingToTarget);
            }
            WanderState::MovingToTarget => {
                if position.translation.distance(movement_goal.target.unwrap()) < 10.0 {
                    *goal = Goals::None;
                }
            }
        }
    }
}
