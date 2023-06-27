use std::time::Duration;

use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    collision, movement,
    reproduction::{self, get_pregnancy_duration, get_reproduction_cooldown},
    rng,
    sim_time::SimTime,
};

use super::Goals;

#[derive(Debug, Clone)]
pub struct WaitingAtRzState {
    waiting_timer: Timer,
    rz: Entity,
}

#[derive(Debug, Clone)]
pub enum ReproducingState {
    FindingRZ,
    MovingToRz(Entity),
    WaitingAtRz(WaitingAtRzState),
}

impl Default for ReproducingState {
    fn default() -> Self {
        Self::FindingRZ
    }
}

pub fn step_reproduce_goal_system(
    mut commands: Commands,
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut rng: ResMut<rng::Rng>,
    mut person: Query<
        (
            Entity,
            &mut Goals,
            &Transform,
            &mut movement::MovementGoal,
            &collision::CollisionHolder,
        ),
        With<Enum!(super::Goals::Reproduce)>,
    >,
    mut reproductive_query: Query<&mut reproduction::Reproductive>,
    reproductive_zones_q: Query<(
        Entity,
        &Transform,
        &collision::CollisionHolder,
        &reproduction::ReproductiveZone,
    )>,
) {
    for (entity, mut goal, trans, mut move_goal, col) in &mut person {
        let reproducing = match goal.clone() {
            Goals::Reproduce(x) => x,
            _ => todo!(),
        };

        match reproducing {
            ReproducingState::FindingRZ => {
                let mut closest_rz: Option<(Entity, Vec3)> = None;
                let mut closest_distance = f32::MAX;

                for (rz_entity, rz_trans, _, _) in &reproductive_zones_q {
                    let distance = trans.translation.distance_squared(rz_trans.translation);

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_rz = Some((rz_entity, rz_trans.translation));
                    }
                }

                if let Some((rz_entity, trans)) = closest_rz {
                    *goal = Goals::Reproduce(ReproducingState::MovingToRz(rz_entity));
                    move_goal.target = Some(trans);
                } else {
                    *goal = Goals::None;
                    continue;
                }
            }
            ReproducingState::MovingToRz(target) => {
                // Check RZ reached
                if col.colliding_with(target).is_some() {
                    *goal = Goals::Reproduce(ReproducingState::WaitingAtRz(WaitingAtRzState {
                        waiting_timer: Timer::new(Duration::from_secs(5), TimerMode::Once),
                        rz: target,
                    }));
                }
            }
            ReproducingState::WaitingAtRz(mut waiting) => {
                waiting.waiting_timer.tick(sim_time.delta(&time));

                if waiting.waiting_timer.finished() {
                    *goal = Goals::None;
                    continue;
                }

                // Check if anyone else has shown up
                let rz_colliders = match reproductive_zones_q
                    .get_component::<collision::CollisionHolder>(waiting.rz)
                {
                    Ok(result) => result,
                    Err(_) => {
                        *goal = Goals::None;
                        continue;
                    }
                };

                // Look though everyone at the RZ
                for event in &rz_colliders.events {
                    if event.other == entity {
                        continue;
                    }

                    {
                        let mut mate_reproductive = match reproductive_query.get_mut(event.other) {
                            Ok(result) => result,
                            Err(_) => continue,
                        };
                        mate_reproductive.reproduction_timer =
                            Timer::new(get_reproduction_cooldown(&mut rng.inner), TimerMode::Once);
                    }

                    let pregnancy_duration = get_pregnancy_duration(&mut rng.inner);

                    {
                        let mut reproductive = reproductive_query.get_mut(entity).unwrap();

                        reproductive.reproduction_timer = Timer::new(
                            pregnancy_duration + get_reproduction_cooldown(&mut rng.inner),
                            TimerMode::Once,
                        );
                    }

                    commands.entity(entity).insert(reproduction::Pregnant {
                        parents: [entity, event.other],
                        pregnancy_timer: Timer::new(pregnancy_duration, TimerMode::Once),
                    });

                    *goal = Goals::None;
                }

                // Since we modify the data
                *goal = Goals::Reproduce(ReproducingState::WaitingAtRz(waiting));
            }
        }
    }
}
