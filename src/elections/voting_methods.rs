#[derive(Debug)]
pub struct OptionRating {
    pub option_index: usize,
    pub rating: i32,
}

pub trait VotingMethod<T> {
    fn fill(option_ratings: &[OptionRating]) -> T;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SingleOptionBallot {
    pub voted_for: usize,
}

impl VotingMethod<SingleOptionBallot> for SingleOptionBallot {
    fn fill(option_ratings: &[OptionRating]) -> SingleOptionBallot {
        let vote_for = option_ratings.iter().next().unwrap().option_index;
        SingleOptionBallot {
            voted_for: vote_for,
        }
    }
}
