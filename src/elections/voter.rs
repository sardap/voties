use bevy::prelude::*;

#[derive(Debug, Component, Clone, Copy)]
pub struct Voter {
    pub money_care: f32,
    pub food_care: f32,
    pub reproductive_care: f32,
}

impl Default for Voter {
    fn default() -> Self {
        Self {
            money_care: 1.0,
            food_care: 1.0,
            reproductive_care: 1.0,
        }
    }
}

impl Voter {
    pub fn new_random(rng: &mut impl rand::Rng) -> Self {
        Self {
            money_care: rng.gen_range(0.5..1.5),
            food_care: rng.gen_range(0.5..1.5),
            reproductive_care: rng.gen_range(0.5..1.5),
        }
    }
}
