use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

use crate::{
    collision,
    elections::election::{Election, VoterAttributes},
    energy::{self, Energy},
    farm,
    hunger::{self, FoodPreferences},
    movement,
    reproduction::{self, Reproductive},
    rng,
};

#[derive(Debug, PartialEq, Clone)]
struct EatingState {
    food: hunger::Food,
    start_time: Duration,
    count: f64,
}

#[derive(Debug, Clone, PartialEq)]
enum HungryState {
    FindingTarget,
    MovingToTarget(Entity),
    Eating(EatingState),
}

impl Default for HungryState {
    fn default() -> Self {
        Self::FindingTarget
    }
}

#[derive(Component, Default)]
pub struct Hungry {
    state: HungryState,
}

pub fn step_hunger_goal_system(
    mut commands: Commands,
    time: Res<Time>,
    mut person: Query<(
        Entity,
        &mut energy::Energy,
        &mut Hungry,
        &Transform,
        Option<&hunger::FoodPreferences>,
        &mut movement::MovementGoal,
        &collision::CollisionHolder,
        &mut hunger::Stomach,
    )>,
    mut farms: Query<(Entity, &Transform, &hunger::Food, &mut farm::Farm)>,
) {
    for (entity, mut energy, mut hungry, position, food_pref, mut move_goal, col, mut stomach) in
        &mut person
    {
        match hungry.state.clone() {
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
                    hungry.state = HungryState::MovingToTarget(farm_entity);
                    move_goal.target = Some(trans);
                } else {
                    commands.entity(entity).remove::<Hungry>();
                    continue;
                }
            }
            HungryState::MovingToTarget(target) => {
                // Farm reached
                if col.events.iter().any(|event| event.other == target) {
                    let (_, _, farm_food, mut farm_farm) = match farms.get_mut(target) {
                        Ok(result) => result,
                        Err(_) => {
                            commands.entity(entity).remove::<Hungry>();
                            continue;
                        }
                    };

                    // Check will eat food
                    if let Some(food_pref) = food_pref {
                        if !food_pref.will_eat(farm_food) {
                            commands.entity(entity).remove::<Hungry>();
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

                    hungry.state = HungryState::Eating(EatingState {
                        food: farm_food.clone(),
                        start_time: time.elapsed(),
                        count: count as f64,
                    });
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
                    commands.entity(entity).remove::<Hungry>();
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct WaitingAtRzState {
    start_time: Duration,
    rz: Entity,
}

#[derive(Debug, Clone, PartialEq)]
enum ReproducingState {
    FindingRZ,
    MovingToRz(Entity),
    WaitingAtRz(WaitingAtRzState),
}

impl Default for ReproducingState {
    fn default() -> Self {
        Self::FindingRZ
    }
}

#[derive(Component, Default)]
pub struct Reproducing {
    state: ReproducingState,
}

pub fn step_reproduce_goal_system(
    mut commands: Commands,
    time: Res<Time>,
    mut person: Query<
        (
            Entity,
            &mut Reproducing,
            &Transform,
            &mut movement::MovementGoal,
            &collision::CollisionHolder,
        ),
        With<reproduction::Reproductive>,
    >,
    mut reproductive_query: Query<&mut reproduction::Reproductive>,
    reproductive_zones_q: Query<(
        Entity,
        &Transform,
        &collision::CollisionHolder,
        &reproduction::ReproductiveZone,
    )>,
) {
    for (entity, mut reproducing, trans, mut move_goal, col) in &mut person {
        match reproducing.state.clone() {
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
                    reproducing.state = ReproducingState::MovingToRz(rz_entity);
                    move_goal.target = Some(trans);
                } else {
                    commands.entity(entity).remove::<Reproducing>();
                    continue;
                }
            }
            ReproducingState::MovingToRz(target) => {
                // Check RZ reached
                if col.events.iter().any(|event| event.other == target) {
                    reproducing.state = ReproducingState::WaitingAtRz(WaitingAtRzState {
                        start_time: time.elapsed(),
                        rz: target,
                    });
                }
            }
            ReproducingState::WaitingAtRz(waiting) => {
                if time.elapsed() - waiting.start_time > Duration::from_secs(5) {
                    commands.entity(entity).remove::<Reproducing>();
                    continue;
                }

                // Check if anyone else has shown up
                let rz_colliders = match reproductive_zones_q
                    .get_component::<collision::CollisionHolder>(waiting.rz)
                {
                    Ok(result) => result,
                    Err(_) => {
                        commands.entity(entity).remove::<Reproducing>();
                        continue;
                    }
                };

                // Look though everyone at the RZ
                for event in &rz_colliders.events {
                    if event.other == entity {
                        continue;
                    }

                    let next_reproduction_time = time.elapsed() + Duration::from_secs(120);
                    {
                        let mut mate_reproductive = match reproductive_query.get_mut(event.other) {
                            Ok(result) => result,
                            Err(_) => continue,
                        };
                        mate_reproductive.next_reproduction = next_reproduction_time;
                    }

                    {
                        let mut reproductive = reproductive_query.get_mut(entity).unwrap();
                        reproductive.next_reproduction = next_reproduction_time;
                    }

                    commands.entity(entity).insert(reproduction::Pregnant {
                        parents: [entity, event.other],
                        pregnant_since: time.elapsed(),
                        pregnancy_duration: Duration::from_secs(10),
                    });

                    commands.entity(entity).remove::<Reproducing>();
                    commands.entity(event.other).remove::<Reproducing>();
                }
            }
        }
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

#[derive(Debug, Component, Default)]
pub struct Wander {
    state: WanderState,
}

pub fn step_wander_goal_system(
    mut commands: Commands,
    mut rng: ResMut<rng::Rng>,
    mut query: Query<(Entity, &mut movement::MovementGoal, &Transform, &mut Wander)>,
) {
    for (entity, mut movement_goal, position, mut wander) in &mut query {
        match wander.state {
            WanderState::FindingTarget => {
                let new_target = Vec3::new(
                    rng.inner.gen_range(-400.0..400.0),
                    rng.inner.gen_range(-300.0..300.0),
                    position.translation.z,
                );
                movement_goal.target = Some(new_target);
                wander.state = WanderState::MovingToTarget;
            }
            WanderState::MovingToTarget => {
                if position.translation.distance(movement_goal.target.unwrap()) < 10.0 {
                    wander.state = WanderState::FindingTarget;
                    commands.entity(entity).remove::<Wander>();
                }
            }
        }
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

#[derive(Debug, Component)]
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
}

pub fn vote_goal_system(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut movement::MovementGoal,
        &Transform,
        &mut Vote,
        Option<&Energy>,
        Option<&FoodPreferences>,
        Option<&Reproductive>,
    )>,
    mut elections: Query<&mut Election>,
) {
    for (entity, mut movement_goal, position, mut vote, energy, food_preferences, reproductive) in
        &mut query
    {
        match vote.state {
            VoteState::FindingVotingCenter => {
                let new_target = Vec3::new(0.0, 0.0, position.translation.z);
                movement_goal.target = Some(new_target);
                vote.state = VoteState::MovingToTarget;
            }
            VoteState::MovingToTarget => {
                if position.translation.distance(movement_goal.target.unwrap()) < 10.0 {
                    vote.state = VoteState::AtVotingCenter;
                }
            }
            VoteState::AtVotingCenter => {
                let mut election = match elections.get_mut(vote.target_election) {
                    Ok(election) => election,
                    Err(_) => {
                        commands.entity(entity).remove::<Vote>();
                        continue;
                    }
                };

                election.vote(
                    entity,
                    VoterAttributes {
                        energy,
                        food_preferences,
                        reproductive,
                    },
                );

                commands.entity(entity).remove::<Vote>();
            }
        }
    }
}
