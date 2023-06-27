use bevy::prelude::*;

use super::election::want_level;

#[derive(Debug, Default, Component, Clone, Copy)]
pub struct Voter {
    pub money_care: i32,
    pub food_care: i32,
    pub reproductive_care: i32,
    pub housing_care: i32,
    pub death_care: i32,
}

fn random_care_value(rng: &mut impl rand::Rng) -> i32 {
    rng.gen_range(want_level::EXTREMELY_NEGATIVE..want_level::EXTREMELY_POSITIVE)
}

impl Voter {
    pub fn new_random(rng: &mut impl rand::Rng) -> Self {
        Self {
            money_care: random_care_value(rng),
            food_care: random_care_value(rng),
            reproductive_care: random_care_value(rng),
            housing_care: random_care_value(rng),
            death_care: random_care_value(rng),
        }
    }
}
