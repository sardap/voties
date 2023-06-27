use super::{
    election::{ElectionOption, ElectionTypeResult},
    voting_methods::{
        fill, vote_bundle, LeastFavoriteSingleOptionBallot, OptionRating, VoteBundle,
    },
};

#[derive(Debug, Clone)]
pub struct VoteTally {
    option_index: usize,
    votes: usize,
}

#[derive(Debug, Clone)]
pub struct AntiPluralityResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub bundles: Vec<VoteBundle<LeastFavoriteSingleOptionBallot>>,
    pub vote_tally: Vec<VoteTally>,
}

pub fn result(
    options: &[ElectionOption],
    option_ratings: &Vec<&Vec<OptionRating>>,
) -> ElectionTypeResult {
    let votes = fill::<LeastFavoriteSingleOptionBallot>(option_ratings);

    let bundles = vote_bundle(&votes);

    let mut vote_tally = vec![];
    for option_index in 0..options.len() {
        let mut tally = 0;
        for bundle in &bundles {
            if bundle.ballot.least_favorite == option_index {
                tally += bundle.votes;
            }
        }
        vote_tally.push(VoteTally {
            option_index,
            votes: tally,
        });
    }

    vote_tally.sort_by(|a, b| a.votes.cmp(&b.votes));

    ElectionTypeResult::AntiPluralityResult(AntiPluralityResult {
        winner: options[vote_tally[0].option_index].clone(),
        total_votes: votes.len(),
        bundles,
        vote_tally,
    })
}
