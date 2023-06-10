use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    elections::election::Election,
    energy::Energy,
    goals::{self, Goals},
    hunger::Stomach,
    reproduction::Reproductive,
};

#[derive(Debug, Component, Default)]
pub struct Brain;

fn do_election_goal(
    goal: &mut Goals,
    entity: Entity,
    elections: &Query<(Entity, &Election)>,
) -> bool {
    for (election_entity, election) in elections {
        if election.voted.contains(&entity) {
            continue;
        }

        *goal = Goals::Vote(goals::Vote::new(election_entity));
        return true;
    }

    false
}

pub fn decide_system(
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut Goals,
            &Energy,
            Option<&Stomach>,
            Option<&Reproductive>,
        ),
        (With<Brain>, With<Enum!(goals::Goals::None)>),
    >,
    elections: Query<(Entity, &Election)>,
) {
    for (entity, mut goal, energy, stomach, reproductive) in &mut query {
        if do_election_goal(&mut goal, entity, &elections) {
            continue;
        }

        if let Some(stomach) = stomach {
            if energy.current_kcal <= 100.0 && stomach.percent_filled() <= 0.8 {
                *goal = Goals::Hungry(default());
                continue;
            }
        }

        if let Some(reproductive) = reproductive {
            if energy.current_kcal >= 1500.0 && reproductive.next_reproduction < time.elapsed() {
                *goal = Goals::Reproduce(default());
                continue;
            }
        }

        *goal = Goals::Wander(default());
    }
}
