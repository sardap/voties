use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{fill, vote_bundle, OptionRating, ScoreBallot, VoteBundle},
};

#[derive(Debug, Clone)]
pub struct RunoffTally {
    option_index: usize,
    votes: usize,
}

#[derive(Debug, Clone)]
pub struct Runoff {
    a: RunoffTally,
    b: RunoffTally,
}

#[derive(Debug, Clone)]
pub struct ScoreTally {
    pub option_index: usize,
    pub score: i32,
}

#[derive(Debug, Clone)]
pub struct StarResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub runoff: Runoff,
    pub score_tally: Vec<ScoreTally>,
    pub bundles: Vec<VoteBundle<ScoreBallot<5>>>,
}

#[derive(Debug, Clone, Default)]
pub struct Star;

impl ElectionImpl for Star {
    fn result(
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult {
        let votes = fill::<ScoreBallot<5>>(option_ratings);

        let bundles = vote_bundle(&votes);

        let mut score_tally = vec![];
        for option_index in 0..options.len() {
            let mut score = 0;
            for bundle in &bundles {
                score += bundle.ballot.votes[option_index] * bundle.votes as i32;
            }
            score_tally.push(ScoreTally {
                option_index,
                score,
            });
        }

        score_tally.sort_by(|a, b| a.score.cmp(&b.score).reverse());

        let a = score_tally[0].option_index;
        let b = score_tally[1].option_index;

        let mut votes_a = 0;
        let mut votes_b = 0;

        for bundle in &bundles {
            let a = bundle.ballot.votes[a];
            let b = bundle.ballot.votes[b];

            if a > b {
                votes_a += bundle.votes;
            } else if b > a {
                votes_b += bundle.votes;
            }
        }

        let winner = if votes_a > votes_b { a } else { b };

        ElectionTypeResult::StarResult(StarResult {
            winner: options[winner].clone(),
            total_votes: votes.len(),
            runoff: Runoff {
                a: RunoffTally {
                    option_index: a,
                    votes: votes_a,
                },
                b: RunoffTally {
                    option_index: b,
                    votes: votes_b,
                },
            },
            score_tally,
            bundles,
        })
    }
}
