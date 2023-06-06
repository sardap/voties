use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{OptionRating, SingleOptionBallot, VotingMethod},
};

#[derive(Debug)]
pub struct FirstPastThePostTally {
    pub option: ElectionOption,
    pub votes: usize,
}

#[derive(Debug)]
pub struct FirstPastThePostResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub tallies: Vec<FirstPastThePostTally>,
}

#[derive(Debug, Clone, Default)]
pub struct FirstPastThePost {
    votes: Vec<SingleOptionBallot>,
}

impl ElectionImpl for FirstPastThePost {
    fn vote(&mut self, option_ratings: &[OptionRating]) {
        self.votes.push(SingleOptionBallot::fill(option_ratings));
    }

    fn result(&self, options: &[ElectionOption]) -> ElectionTypeResult {
        let mut tallies = options
            .iter()
            .enumerate()
            .map(|(option_index, option)| FirstPastThePostTally {
                option: option.clone(),
                votes: self
                    .votes
                    .iter()
                    .filter(|vote| vote.voted_for == option_index)
                    .count(),
            })
            .collect::<Vec<_>>();

        tallies.sort_by(|a, b| b.votes.cmp(&a.votes));

        ElectionTypeResult::FirstPastThePostResult(FirstPastThePostResult {
            winner: tallies.first().unwrap().option.clone(),
            total_votes: self.votes.len(),
            tallies,
        })
    }
}
