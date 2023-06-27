use std::collections::{HashMap, HashSet};

use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{
        fill, vote_bundle, MandatoryPreferentialBallot, OptionRating, VoteBundle, VoteCount,
    },
};

#[derive(Debug, Clone)]
pub struct Round {
    pub eliminated_candidates: HashSet<usize>,
    pub vote_tally: Vec<VoteCount>,
}

#[derive(Debug, Clone)]
pub struct PreferentialResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub rounds: Vec<Round>,
    pub bundles: Vec<VoteBundle<MandatoryPreferentialBallot>>,
}

#[derive(Debug, Clone, Default)]
pub struct Preferential;

impl ElectionImpl for Preferential {
    fn result(
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult {
        let votes = fill::<MandatoryPreferentialBallot>(option_ratings);

        let fifty_percent = (votes.len() / 2) as i64;
        let bundles = vote_bundle(&votes);

        let mut rounds = Vec::new();
        let mut eliminated_candidates = HashSet::new();
        loop {
            let mut vote_tally = HashMap::<usize, i64>::new();
            for i in 0..options.len() {
                vote_tally.insert(i, 0);
            }

            for bundle in &bundles {
                for option_index in &bundle.ballot.votes {
                    if !eliminated_candidates.contains(option_index) {
                        *vote_tally.get_mut(option_index).unwrap() += bundle.votes as i64;
                        break;
                    }
                }
            }

            let mut vote_tally = vote_tally
                .into_iter()
                .filter(|(k, _)| !eliminated_candidates.contains(k))
                .map(|(k, v)| VoteCount {
                    option: options[k].clone(),
                    votes: v,
                })
                .collect::<Vec<_>>();

            vote_tally.sort_by(|a, b| b.votes.cmp(&a.votes));

            let top = vote_tally[0].clone();
            let bottom = vote_tally.iter().last().unwrap().clone();

            rounds.push(Round {
                eliminated_candidates: eliminated_candidates.clone(),
                vote_tally,
            });

            if top.votes > fifty_percent {
                break;
            }

            if eliminated_candidates.len() == options.len() - 1 {
                break;
            }

            eliminated_candidates.insert(options.iter().position(|o| o == &bottom.option).unwrap());
        }

        ElectionTypeResult::PreferentialResult(PreferentialResult {
            winner: rounds.last().unwrap().vote_tally[0].option.clone(),
            total_votes: votes.len(),
            rounds,
            bundles,
        })
    }
}
