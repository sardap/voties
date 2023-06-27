pub mod eating;
pub mod find_housing;
pub mod reproducing;
pub mod vote;
pub mod wander;

use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::sets::LifeSet;

use self::{
    eating::{step_hunger_goal_system, HungryState},
    find_housing::{step_find_housing_goal_system, HousingState},
    reproducing::{step_reproduce_goal_system, ReproducingState},
    vote::{vote_goal_system, Vote},
    wander::{step_wander_goal_system, WanderState},
};

#[derive(Debug, Clone, Component, EnumFilter)]
pub enum Goals {
    None,
    Hungry(HungryState),
    Reproduce(ReproducingState),
    Wander(WanderState),
    Vote(Vote),
    FindHousing(HousingState),
}

#[derive(Debug)]
pub struct GoalsPlugin;

impl Plugin for GoalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            step_hunger_goal_system.in_set(LifeSet::Goal),
            step_reproduce_goal_system.in_set(LifeSet::Goal),
            step_wander_goal_system.in_set(LifeSet::Goal),
            vote_goal_system.in_set(LifeSet::Goal),
            step_find_housing_goal_system.in_set(LifeSet::Goal),
        ));
    }
}
