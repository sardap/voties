use super::{
    election::{ElectionImpl, ElectionOption, ElectionTypeResult},
    voting_methods::{fill, vote_bundle, GoodBadOkBallot, GoodOkBad, OptionRating, VoteBundle},
};

#[derive(Debug, Clone)]
pub struct GoodOkBadTally {
    option_index: usize,
    good: usize,
    ok: usize,
    bad: usize,
}

#[derive(Debug, Clone)]
pub struct RunoffTally {
    option_index: usize,
    votes: usize,
}

#[derive(Debug, Clone)]
pub struct Runoff {
    pub a: RunoffTally,
    pub b: RunoffTally,
}

#[derive(Debug, Clone)]
pub struct GoodOkBadResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub tally: Vec<GoodOkBadTally>,
    pub runoff: Runoff,
    pub bundles: Vec<VoteBundle<GoodBadOkBallot>>,
}

#[derive(Debug, Clone, Default)]
pub struct GoodOkBadElection;

impl ElectionImpl for GoodOkBadElection {
    fn result(
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult {
        let votes = fill::<GoodBadOkBallot>(option_ratings);

        let bundles = vote_bundle(&votes);

        let mut vote_tally = vec![];

        for option_index in 0..options.len() {
            let mut tally = GoodOkBadTally {
                option_index,
                good: 0,
                ok: 0,
                bad: 0,
            };
            for bundle in &bundles {
                match bundle.ballot.votes[option_index] {
                    GoodOkBad::Good => tally.good += bundle.votes,
                    GoodOkBad::Ok => tally.ok += bundle.votes,
                    GoodOkBad::Bad => tally.bad += bundle.votes,
                }
            }
            vote_tally.push(tally);
        }

        // Sort by Good
        vote_tally.sort_by(|a, b| a.good.cmp(&b.good).reverse());

        let mut top_three = vec![&vote_tally[0], &vote_tally[1], &vote_tally[2]];
        top_three.sort_by(|a, b| a.bad.cmp(&b.bad));

        let top_two = vec![top_three[0], top_three[1]];

        let mut votes_a = 0;
        let mut votes_b = 0;

        for bundle in &bundles {
            let a = bundle.ballot.votes[top_two[0].option_index];
            let b = bundle.ballot.votes[top_two[1].option_index];

            if a > b {
                votes_a += bundle.votes;
            } else if b > a {
                votes_b += bundle.votes;
            }
        }

        let runoff = Runoff {
            a: RunoffTally {
                option_index: top_two[0].option_index,
                votes: votes_a,
            },
            b: RunoffTally {
                option_index: top_two[1].option_index,
                votes: votes_b,
            },
        };

        let winner = if votes_a > votes_b {
            top_two[0].option_index
        } else {
            top_two[1].option_index
        };

        ElectionTypeResult::GoodOkBadResult(GoodOkBadResult {
            winner: options[winner].clone(),
            total_votes: votes.len(),
            tally: vote_tally,
            runoff,
            bundles,
        })
    }
}
