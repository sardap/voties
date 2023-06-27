use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{buildings::farm, collision, energy, hunger, movement, sim_time::SimTime};

use super::Goals;

#[derive(Debug, Clone)]
pub struct EatingState {
    food: hunger::Food,
    waiting: Timer,
    count: f64,
}

#[derive(Debug, Clone)]
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

pub fn step_hunger_goal_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
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
        With<Enum!(super::Goals::Hungry)>,
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
                if col.colliding_with(target).is_some() {
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
                        count: count as f64,
                        waiting: Timer::new(farm_food.how_long_to_eat(), TimerMode::Once),
                    }));
                }
            }
            HungryState::Eating(mut eating_state) => {
                if eating_state
                    .waiting
                    .tick(sim_time.delta(&time))
                    .just_finished()
                {
                    stomach.filled_ml += eating_state.food.ml * eating_state.count;
                    energy.current_kcal += eating_state.food.kcal * eating_state.count;
                    *goal = Goals::None;
                } else {
                    *goal = Goals::Hungry(HungryState::Eating(eating_state));
                }
            }
        }
    }
}
