use std::collections::HashMap;

use super::election::{want_level, ElectionOption};

#[derive(Debug, Clone)]
pub struct OptionRating {
    pub option_index: usize,
    pub rating: i32,
}

#[derive(Debug, Clone)]
pub struct VoteBundle<T> {
    pub ballot: T,
    pub votes: usize,
}

pub trait VotingMethod {
    fn fill(option_ratings: &[OptionRating]) -> Self;
}

pub fn fill<T>(option_ratings: &Vec<&Vec<OptionRating>>) -> Vec<T>
where
    T: VotingMethod,
{
    option_ratings
        .iter()
        .map(|option_rating| T::fill(option_rating))
        .collect::<Vec<_>>()
}

pub fn vote_bundle<T>(votes: &[T]) -> Vec<VoteBundle<T>>
where
    T: VotingMethod,
    T: std::hash::Hash,
    T: std::cmp::Eq,
    T: std::cmp::PartialEq,
    T: std::clone::Clone,
{
    let mut tallies: HashMap<T, usize> = HashMap::new();

    for vote in votes {
        if tallies.get_mut(vote).is_none() {
            tallies.insert(vote.clone(), 0);
        }

        let tally = tallies.get_mut(vote).unwrap();
        *tally += 1;
    }

    let mut tallies: Vec<_> = tallies
        .into_iter()
        .map(|(k, v)| VoteBundle {
            ballot: k,
            votes: v,
        })
        .collect();

    tallies.sort_by(|a, b| b.votes.cmp(&a.votes));

    tallies
}

#[derive(Debug, Clone)]
pub struct VoteCount {
    pub option: ElectionOption,
    pub votes: i64,
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SingleOptionBallot {
    pub voted_for: usize,
}

impl VotingMethod for SingleOptionBallot {
    fn fill(option_ratings: &[OptionRating]) -> SingleOptionBallot {
        let vote_for = option_ratings.iter().next().unwrap().option_index;
        SingleOptionBallot {
            voted_for: vote_for,
        }
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct MultipleOptionBallot {
    pub voted_for: Vec<usize>,
}

impl VotingMethod for MultipleOptionBallot {
    fn fill(option_ratings: &[OptionRating]) -> Self {
        let mut voting_for = vec![];
        for option_rating in option_ratings {
            if option_rating.rating >= want_level::SLIGHTLY_POSITIVE {
                voting_for.push(option_rating.option_index);
            }
        }

        MultipleOptionBallot {
            voted_for: voting_for,
        }
    }
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct MandatoryPreferentialBallot {
    pub votes: Vec<usize>,
}

impl VotingMethod for MandatoryPreferentialBallot {
    fn fill(option_ratings: &[OptionRating]) -> Self {
        MandatoryPreferentialBallot {
            // already in order
            votes: option_ratings.iter().map(|i| i.option_index).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
pub enum GoodOkBad {
    Bad,
    Ok,
    Good,
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct GoodBadOkBallot {
    pub votes: Vec<GoodOkBad>,
}

impl VotingMethod for GoodBadOkBallot {
    fn fill(option_ratings: &[OptionRating]) -> Self {
        let mut votes = vec![GoodOkBad::Bad; option_ratings.len()];
        for option_rating in option_ratings {
            votes[option_rating.option_index] = if option_rating.rating >= want_level::POSITIVE {
                GoodOkBad::Good
            } else if option_rating.rating >= want_level::SLIGHTLY_NEGATIVE {
                GoodOkBad::Ok
            } else {
                GoodOkBad::Bad
            }
        }

        GoodBadOkBallot { votes }
    }
}

pub type Score = i32;

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq)]
pub struct ScoreBallot<const N: usize> {
    pub votes: Vec<Score>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
struct Threshold {
    rating: Score,
    score: Score,
}

const fn get_thresholds_scores<const N: usize>() -> [Threshold; N + 1] {
    let mut result = [Threshold {
        rating: 0,
        score: 0,
    }; N + 1];

    const LEVELS: [Score; 7] = [
        want_level::EXTREMELY_NEGATIVE,
        want_level::NEGATIVE,
        want_level::SLIGHTLY_NEGATIVE,
        want_level::NEUTRAL,
        want_level::SLIGHTLY_POSITIVE,
        want_level::POSITIVE,
        want_level::EXTREMELY_POSITIVE,
    ];

    let mut i = 0;
    while i < N + 1 {
        let level_percent = i as f64 / (N + 1) as f64;
        let index = ((LEVELS.len() - 1) as f64 * level_percent) as usize;
        let rating = LEVELS[index];

        result[i].rating = rating;
        result[i].score = i as Score;

        i += 1;
    }

    // Reverse the array so that the highest score is first
    let mut start = 0;
    let mut end = N;
    while start < end {
        let temp = result[start];
        result[start] = result[end];
        result[end] = temp;
        start += 1;
        end -= 1;
    }

    result
}

impl<const N: usize> VotingMethod for ScoreBallot<N>
where
    [(); N + 1]:,
{
    fn fill(option_ratings: &[OptionRating]) -> Self {
        let scores = get_thresholds_scores::<N>();

        let mut votes = vec![0; option_ratings.len()];
        for option_rating in option_ratings {
            let mut score = 0;
            for threshold in &scores {
                if option_rating.rating >= threshold.rating {
                    score = threshold.score;
                    break;
                }
            }

            votes[option_rating.option_index] = score;
        }

        ScoreBallot { votes }
    }
}

#[derive(Debug, Clone)]
pub struct ScoreCount<const N: usize> {
    pub option_index: usize,
    pub scores: Vec<Score>,
    pub tallies: Vec<usize>,
}

impl<const N: usize> ScoreCount<N> {
    pub fn median(&self) -> Score {
        self.scores[self.scores.len() / 2]
    }

    pub fn majority(&self) -> Score {
        let vote_count = self.scores.len() as f64;
        let mut sum = 0.0;
        for (i, rating) in self.tallies.iter().enumerate() {
            sum += (*rating as f64) / vote_count;
            if sum >= 0.5 {
                return i as Score;
            }
        }

        panic!("Impossible");
    }

    pub fn percent_for_score(&self, score: Score) -> f64 {
        let total = self.scores.len();
        let count = self.tallies[score as usize];

        count as f64 / total as f64
    }

    pub fn percent_above_grade(&self, target_grade: Score) -> f64 {
        ((target_grade as usize)..N)
            .map(|i| self.percent_for_score(i as Score))
            .sum()
    }

    pub fn percent_below_grade(&self, target_grade: Score) -> f64 {
        (0..(target_grade as usize))
            .rev()
            .map(|i| self.percent_for_score(i as Score))
            .sum()
    }
}

pub fn get_score_counts<const N: usize>(votes: &[ScoreBallot<N>]) -> Vec<ScoreCount<N>> {
    if votes.len() == 0 {
        return vec![];
    }

    // Get number of candidates
    let num_candidates = votes[0].votes.len();

    let mut result = (0..num_candidates)
        .map(|i| ScoreCount {
            option_index: i,
            scores: Default::default(),
            tallies: vec![0; N + 1],
        })
        .collect::<Vec<_>>();

    for vote in votes {
        for (i, score) in vote.votes.iter().enumerate() {
            let count = result.get_mut(i).unwrap();
            count.scores.push(*score);
            count.tallies[*score as usize] += 1;
        }
    }

    // sort each of the scores
    for score_count in result.iter_mut() {
        score_count.scores.sort();
    }

    result
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct LeastFavoriteSingleOptionBallot {
    pub least_favorite: usize,
}

impl VotingMethod for LeastFavoriteSingleOptionBallot {
    fn fill(option_ratings: &[OptionRating]) -> Self {
        let vote_for = option_ratings.iter().last().unwrap().option_index;
        Self {
            least_favorite: vote_for,
        }
    }
}
