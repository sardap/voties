use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    buildings::{self, house::House},
    collision, movement,
    shelter::RequiresHouse,
};

use super::Goals;

#[derive(Debug, Clone, Copy)]
pub enum HousingState {
    FindingHouse,
    MovingToHouse(Entity),
}

impl Default for HousingState {
    fn default() -> Self {
        HousingState::FindingHouse
    }
}

pub fn step_find_housing_goal_system(
    mut query: Query<
        (
            Entity,
            &mut Goals,
            &mut movement::MovementGoal,
            &Transform,
            &collision::CollisionHolder,
            &mut RequiresHouse,
        ),
        With<Enum!(super::Goals::FindHousing)>,
    >,
    mut housing: Query<
        (Entity, &Transform, &mut House),
        With<Enum!(buildings::building::BuildingStatus::Operational)>,
    >,
) {
    for (entity, mut goal, mut movement_goal, position, col_holder, mut requires_house) in
        &mut query
    {
        let state = match goal.clone() {
            Goals::FindHousing(x) => x,
            _ => todo!(),
        };

        match state {
            HousingState::FindingHouse => {
                let mut closest_house: Option<(Entity, Vec3)> = None;
                let mut closest_distance = f32::MAX;

                for (entity, transform, house) in &housing {
                    if house.is_full() {
                        continue;
                    }

                    let distance = position.translation.distance_squared(transform.translation);

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_house = Some((entity, transform.translation));
                    }
                }

                if let Some((entity, trans)) = closest_house {
                    *goal = Goals::FindHousing(HousingState::MovingToHouse(entity));
                    movement_goal.target = Some(trans);
                } else {
                    *goal = Goals::None;
                    continue;
                }
            }
            HousingState::MovingToHouse(target) => {
                if col_holder.colliding_with(target).is_some() {
                    if let Ok((_, _, mut house)) = housing.get_mut(target) {
                        if let Ok(_) = house.add_occupant(entity) {
                            requires_house.shelter = Some(target);
                        }
                    }

                    *goal = Goals::None;
                }
            }
        }
    }
}
