use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{fill, vote_bundle, OptionRating, SingleOptionBallot, VoteBundle},
};

#[derive(Debug)]
pub struct FirstPastThePostTally {
    pub option: ElectionOption,
    pub votes: usize,
}

#[derive(Debug, Clone)]
pub struct FirstPastThePostResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub vote_bundle: Vec<VoteBundle<SingleOptionBallot>>,
}

#[derive(Debug, Clone, Default)]
pub struct FirstPastThePost;

impl ElectionImpl for FirstPastThePost {
    fn result(
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult {
        let votes = fill::<SingleOptionBallot>(option_ratings);

        let bundles = vote_bundle(&votes);

        ElectionTypeResult::FirstPastThePostResult(FirstPastThePostResult {
            winner: options[bundles.iter().next().unwrap().ballot.voted_for].clone(),
            vote_bundle: bundles,
            total_votes: votes.len(),
        })
    }
}
