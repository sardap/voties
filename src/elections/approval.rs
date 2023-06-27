use std::collections::HashMap;

use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{
        fill, vote_bundle, MultipleOptionBallot, OptionRating, VoteBundle, VoteCount,
    },
};

#[derive(Debug, Clone)]
pub struct ApprovalResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub vote_count: Vec<VoteCount>,
    pub bundles: Vec<VoteBundle<MultipleOptionBallot>>,
}

#[derive(Debug, Clone, Default)]
pub struct Approval;

impl ElectionImpl for Approval {
    fn result(
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult {
        let votes = fill::<MultipleOptionBallot>(option_ratings);

        let bundles = vote_bundle(&votes);

        let mut vote_count = HashMap::<usize, i64>::new();
        for i in 0..options.len() {
            vote_count.insert(i, 0);
        }

        for bundle in &bundles {
            for vote in &bundle.ballot.voted_for {
                let count = vote_count.get_mut(vote).unwrap();
                *count += 1;
            }
        }

        let mut vote_count: Vec<_> = vote_count
            .into_iter()
            .map(|(k, v)| VoteCount {
                option: options[k].clone(),
                votes: v,
            })
            .collect();

        vote_count.sort_by(|a, b| b.votes.cmp(&a.votes));

        ElectionTypeResult::ApprovalResult(ApprovalResult {
            winner: vote_count.iter().next().unwrap().option.clone(),
            total_votes: votes.len(),
            vote_count,
            bundles,
        })
    }
}
