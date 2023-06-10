use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{vote_bundle, OptionRating, SingleOptionBallot, VoteBundle, VotingMethod},
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
    pub vote_bundle: Vec<VoteBundle<SingleOptionBallot>>,
}

#[derive(Debug, Clone, Default)]
pub struct FirstPastThePost {
    pub votes: Vec<SingleOptionBallot>,
}

impl ElectionImpl for FirstPastThePost {
    fn vote(&mut self, option_ratings: &[OptionRating]) {
        self.votes.push(SingleOptionBallot::fill(option_ratings));
    }

    fn result(&self, options: &[ElectionOption]) -> ElectionTypeResult {
        let bundles = vote_bundle(&self.votes);

        ElectionTypeResult::FirstPastThePostResult(FirstPastThePostResult {
            winner: options[bundles.iter().next().unwrap().ballot.voted_for].clone(),
            vote_bundle: bundles,
            total_votes: self.votes.len(),
        })
    }
}
