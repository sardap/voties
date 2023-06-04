use bevy::prelude::*;

use crate::{
    elections::election::Election, energy::Energy, goals, hunger::Stomach,
    reproduction::Reproductive,
};

#[derive(Debug, Component, Default)]
pub struct Brain;

fn do_election_goal(
    commands: &mut Commands,
    entity: Entity,
    elections: &Query<(Entity, &Election)>,
) -> bool {
    for (election_entity, election) in elections {
        if election.voted.contains(&entity) {
            continue;
        }

        commands
            .entity(entity)
            .insert(goals::Vote::new(election_entity));
        return true;
    }

    false
}

pub fn decide_system(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<
        (Entity, &Energy, Option<&Stomach>, Option<&Reproductive>),
        (
            With<Brain>,
            Without<goals::Reproducing>,
            Without<goals::Hungry>,
            Without<goals::Wander>,
            Without<goals::Vote>,
        ),
    >,
    elections: Query<(Entity, &Election)>,
) {
    for (entity, energy, stomach, reproductive) in &query {
        if do_election_goal(&mut commands, entity, &elections) {
            continue;
        }

        if let Some(stomach) = stomach {
            if energy.current_kcal <= 100.0 && stomach.percent_filled() <= 0.8 {
                commands.entity(entity).insert(goals::Hungry::default());
                continue;
            }
        }

        if let Some(reproductive) = reproductive {
            if energy.current_kcal >= 1500.0 && reproductive.next_reproduction < time.elapsed() {
                commands
                    .entity(entity)
                    .insert(goals::Reproducing::default());
                continue;
            }
        }

        commands.entity(entity).insert(goals::Wander::default());
    }
}
