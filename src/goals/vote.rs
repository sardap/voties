use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    buildings::voting_center::VotingCenter,
    elections::{
        election::{Election, VoterAttributes},
        voter::Voter,
    },
    energy::Energy,
    hunger::{FoodPreferences, Stomach},
    movement,
    reproduction::Reproductive,
    rng,
    shelter::RequiresHouse,
    world_stats,
};

use super::Goals;

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

pub fn vote_goal_system(
    stats: Res<world_stats::WorldStats>,
    mut rng: ResMut<rng::Rng>,
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
            Option<&Stomach>,
            Option<&RequiresHouse>,
        ),
        With<Enum!(super::Goals::Vote)>,
    >,
    voting_centers: Query<&Transform, With<VotingCenter>>,
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
        stomach,
        requires_house,
    ) in &mut query
    {
        let vote = match goal.clone() {
            Goals::Vote(x) => x,
            _ => todo!(),
        };

        match vote.state {
            VoteState::FindingVotingCenter => {
                let mut closest_voting_center: Option<Vec3> = None;
                let mut closest_distance = f32::MAX;

                for transform in &voting_centers {
                    let distance = position.translation.distance_squared(transform.translation);

                    if distance < closest_distance {
                        closest_distance = distance;
                        closest_voting_center = Some(transform.translation);
                    }
                }

                if let Some(trans) = closest_voting_center {
                    *goal = Goals::Vote(vote.new_state(VoteState::MovingToTarget));
                    movement_goal.target =
                        Some(Vec3::new(trans.x, trans.y, position.translation.z));
                } else {
                    *goal = Goals::None;
                    continue;
                }
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
                    &mut rng.inner,
                    entity,
                    VoterAttributes {
                        voter,
                        energy,
                        stomach,
                        food_preferences,
                        reproductive,
                        housing: requires_house,
                    },
                    &stats,
                );

                *goal = Goals::None;
            }
        }
    }
}
