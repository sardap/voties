use std::time::Duration;

use bevy::prelude::*;
use bevy_enum_filter::prelude::*;
use rand::Rng;

use crate::{
    collision,
    elections::{
        election::{Election, VoterAttributes},
        voter::Voter,
    },
    energy::{self, Energy},
    farm,
    hunger::{self, FoodPreferences},
    movement,
    reproduction::{self, Reproductive},
    rng, world_stats,
};

#[derive(Debug, PartialEq, Clone)]
pub struct EatingState {
    food: hunger::Food,
    start_time: Duration,
    count: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HungryState {
    FindingTarget,
    MovingToTarget(Entity),
    Eating(EatingState),
}

impl Default for HungryState {
    fn default() -> Self {
        Self::FindingTarget
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaitingAtRzState {
    start_time: Duration,
    rz: Entity,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy)]
pub enum VoteState {
    FindingVotingCenter,
    MovingToTarget,
    AtVotingCenter,
}

impl Default for VoteState {
    fn default() -> Self {
        VoteState::FindingVotingCenter
    }
}

#[derive(Debug, Clone)]
pub struct Vote {
    state: VoteState,
    target_election: Entity,
}

impl Vote {
    pub fn new(target_election: Entity) -> Self {
        Self {
            state: VoteState::FindingVotingCenter,
            target_election,
        }
    }

    pub fn new_state(&self, state: VoteState) -> Self {
        Self {
            state,
            target_election: self.target_election,
        }
    }
}

#[derive(Debug, Clone, Component, EnumFilter)]
pub enum Goals {
    None,
    Hungry(HungryState),
    Reproduce(ReproducingState),
    Wander(WanderState),
    Vote(Vote),
}

pub fn step_hunger_goal_system(
    time: Res<Time>,
    mut person: Query<
        (
            &mut Goals,
            &mut energy::Energy,
            &Transform,
            Option<&hunger::FoodPreferences>,
            &mut movement::MovementGoal,
            &collision::CollisionHolder,
            &mut hunger::Stomach,
        ),
        With<Enum!(Goals::Hungry)>,
    >,
    mut farms: Query<(Entity, &Transform, &hunger::Food, &mut farm::Farm)>,
) {
    for (mut goal, mut energy, position, food_pref, mut move_goal, col, mut stomach) in &mut person
    {
        let hungry = match goal.clone() {
            Goals::Hungry(hungry) => hungry,
            _ => todo!(),
        };

        match hungry {
            HungryState::FindingTarget => {
                let mut closest_farm: Option<(Entity, Vec3)> = None;
                let mut closest_distance = f32::MAX;

                for (farm_entity, farm_position, food, farm) in &farms {
                    if !stomach.can_eat(food) {
                        continue;
                    }

                    if !farm.has_surplus() {
                        continue;
                    }

                    if let Some(food_pref) = food_pref {
                        if !food_pref.will_eat(food) {
                            continue;
                        }
                    }

                    let distance = position
                        .translation
                        .distance_squared(farm_position.translation);

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_farm = Some((farm_entity, farm_position.translation));
                    }
                }

                if let Some((farm_entity, trans)) = closest_farm {
                    *goal = Goals::Hungry(HungryState::MovingToTarget(farm_entity));
                    move_goal.target = Some(trans);
                } else {
                    *goal = Goals::None;
                    continue;
                }
            }
            HungryState::MovingToTarget(target) => {
                // Farm reached
                if col.events.iter().any(|event| event.other == target) {
                    let (_, _, farm_food, mut farm_farm) = match farms.get_mut(target) {
                        Ok(result) => result,
                        Err(_) => {
                            *goal = Goals::None;
                            continue;
                        }
                    };

                    // Check will eat food
                    if let Some(food_pref) = food_pref {
                        if !food_pref.will_eat(farm_food) {
                            *goal = Goals::None;
                            continue;
                        }
                    }

                    let mut count = 0;
                    while energy.current_kcal + farm_food.kcal * (count as f64) < energy.max_kcal
                        && stomach.filled_ml + farm_food.ml * (count as f64) < stomach.max_size_ml
                        && farm_farm.take_food()
                    {
                        count += 1;
                    }

                    *goal = Goals::Hungry(HungryState::Eating(EatingState {
                        food: farm_food.clone(),
                        start_time: time.elapsed(),
                        count: count as f64,
                    }));
                }
            }
            HungryState::Eating(eating_state) => {
                let time_taken_to_eat = eating_state
                    .food
                    .how_long_to_eat()
                    .mul_f64(eating_state.count);
                if time.elapsed() - eating_state.start_time > time_taken_to_eat {
                    stomach.filled_ml += eating_state.food.ml * eating_state.count;
                    energy.current_kcal += eating_state.food.kcal * eating_state.count;
                    *goal = Goals::None;
                }
            }
        }
    }
}

pub fn step_reproduce_goal_system(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<rng::Rng>,
    mut person: Query<
        (
            Entity,
            &mut Goals,
            &Transform,
            &mut movement::MovementGoal,
            &collision::CollisionHolder,
        ),
        With<Enum!(Goals::Reproduce)>,
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
                if col.events.iter().any(|event| event.other == target) {
                    *goal = Goals::Reproduce(ReproducingState::WaitingAtRz(WaitingAtRzState {
                        start_time: time.elapsed(),
                        rz: target,
                    }));
                }
            }
            ReproducingState::WaitingAtRz(waiting) => {
                if time.elapsed() - waiting.start_time > Duration::from_secs(5) {
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

                    let next_reproduction_time = time.elapsed() + Duration::from_secs(30);
                    {
                        let mut mate_reproductive = match reproductive_query.get_mut(event.other) {
                            Ok(result) => result,
                            Err(_) => continue,
                        };
                        mate_reproductive.next_reproduction = next_reproduction_time
                            + Duration::from_secs(rng.inner.gen_range(0..30));
                    }

                    {
                        let mut reproductive = reproductive_query.get_mut(entity).unwrap();
                        reproductive.next_reproduction = next_reproduction_time
                            + Duration::from_secs(rng.inner.gen_range(0..30));
                    }

                    commands.entity(entity).insert(reproduction::Pregnant {
                        parents: [entity, event.other],
                        pregnant_since: time.elapsed(),
                        pregnancy_duration: Duration::from_secs(10),
                    });

                    *goal = Goals::None;
                    // commands.entity(event.other).remove::<Reproducing>();
                }
            }
        }
    }
}

pub fn step_wander_goal_system(
    mut rng: ResMut<rng::Rng>,
    mut query: Query<
        (&mut Goals, &mut movement::MovementGoal, &Transform),
        With<Enum!(Goals::Wander)>,
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

pub fn vote_goal_system(
    stats: Res<world_stats::WorldStats>,
    mut query: Query<
        (
            Entity,
            &mut Goals,
            &mut movement::MovementGoal,
            &Transform,
            &Voter,
            Option<&Energy>,
            Option<&FoodPreferences>,
            Option<&Reproductive>,
        ),
        With<Enum!(Goals::Vote)>,
    >,
    mut elections: Query<&mut Election>,
) {
    for (
        entity,
        mut goal,
        mut movement_goal,
        position,
        voter,
        energy,
        food_preferences,
        reproductive,
    ) in &mut query
    {
        let vote = match goal.clone() {
            Goals::Vote(x) => x,
            _ => todo!(),
        };

        match vote.state {
            VoteState::FindingVotingCenter => {
                let new_target = Vec3::new(0.0, 0.0, position.translation.z);
                movement_goal.target = Some(new_target);
                *goal = Goals::Vote(vote.new_state(VoteState::MovingToTarget));
            }
            VoteState::MovingToTarget => {
                if position.translation.distance(movement_goal.target.unwrap()) < 10.0 {
                    *goal = Goals::Vote(vote.new_state(VoteState::AtVotingCenter));
                }
            }
            VoteState::AtVotingCenter => {
                let mut election = match elections.get_mut(vote.target_election) {
                    Ok(election) => election,
                    Err(_) => {
                        *goal = Goals::None;
                        continue;
                    }
                };

                election.vote(
                    entity,
                    VoterAttributes {
                        voter,
                        energy,
                        food_preferences,
                        reproductive,
                    },
                    &stats,
                );

                *goal = Goals::None;
            }
        }
    }
}
