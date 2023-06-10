use std::collections::HashMap;

use super::election::ElectionOption;

#[derive(Debug)]
pub struct OptionRating {
    pub option_index: usize,
    pub rating: i32,
}

#[derive(Debug)]
pub struct VoteBundle<T> {
    pub ballot: T,
    pub votes: usize,
}

pub trait VotingMethod {
    fn fill(option_ratings: &[OptionRating]) -> Self;
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
    fn fill(option_ratings: &[OptionRating]) -> MultipleOptionBallot {
        let max = (option_ratings
            .iter()
            .max_by(|a, b| a.rating.cmp(&b.rating))
            .unwrap()
            .rating as f64
            * 0.8) as i32;

        let mut voting_for = vec![];
        for option_rating in option_ratings {
            if option_rating.rating >= max {
                voting_for.push(option_rating.option_index);
            }
        }

        MultipleOptionBallot {
            voted_for: voting_for,
        }
    }
}
