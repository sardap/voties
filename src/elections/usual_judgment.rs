use crate::elections::voting_methods::{get_score_counts, ScoreBallot};

use super::{
    election::{ElectionOption, ElectionTypeResult},
    voting_methods::{fill, vote_bundle, OptionRating, ScoreCount, VoteBundle},
};

type UsualJudgmentBallot = ScoreBallot<6>;
type UsualJudgmentCount = ScoreCount<6>;

#[derive(Debug, Clone)]
pub struct UsualJudgmentResult {
    pub winner: ElectionOption,
    pub total_votes: usize,
    pub bundles: Vec<VoteBundle<UsualJudgmentBallot>>,
    pub counts: Vec<UsualJudgmentCount>,
}

pub fn get_highest_majority_grade(counts: &[UsualJudgmentCount]) -> Vec<&UsualJudgmentCount> {
    let max = counts.iter().map(|count| count.majority()).max().unwrap();

    counts.iter().filter(|i| i.majority() == max).collect()
}

fn score_n(count: &UsualJudgmentCount, n: f64) -> f64 {
    let majority_grade = count.majority();
    let a: f64 = majority_grade.into();
    let p: f64 = count.percent_above_grade(majority_grade).powf(n);
    let q: f64 = count.percent_below_grade(majority_grade).powf(n);

    let p_minus_q = p - q;

    let n_a = a + 0.5 * (p_minus_q / (1.0 - p_minus_q));
    let deviation = if n_a < a { -(a - n_a) } else { a - n_a };

    deviation
}

pub fn result(
    options: &[ElectionOption],
    option_ratings: &Vec<&Vec<OptionRating>>,
) -> ElectionTypeResult {
    let votes = fill::<UsualJudgmentBallot>(option_ratings);

    let bundles = vote_bundle(&votes);

    let counts = get_score_counts(&votes);

    let highest_majority_grade = get_highest_majority_grade(&counts);
    if highest_majority_grade.len() == 1 {
        return ElectionTypeResult::UsualJudgment(UsualJudgmentResult {
            winner: options[highest_majority_grade[0].option_index].clone(),
            total_votes: votes.len(),
            bundles,
            counts,
        });
    }

    let mut finalists = highest_majority_grade;

    struct Tiebreaker {
        option_index: usize,
        score: f64,
    }

    // Multiple share majority grade
    for i in 0..50 {
        let scores = finalists
            .iter()
            .map(|count| Tiebreaker {
                option_index: count.option_index,
                score: score_n(count, i as f64),
            })
            .collect::<Vec<_>>();

        let best_score = scores
            .iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap())
            .unwrap()
            .score;

        for i in &scores {
            if i.score != best_score {
                finalists.remove(
                    finalists
                        .iter()
                        .position(|j| j.option_index == i.option_index)
                        .unwrap(),
                );
            }
        }

        if i == 49 || finalists.len() == 1 {
            return ElectionTypeResult::UsualJudgment(UsualJudgmentResult {
                winner: options[finalists[0].option_index].clone(),
                total_votes: votes.len(),
                bundles,
                counts,
            });
        }
    }

    panic!("impossible")
}
