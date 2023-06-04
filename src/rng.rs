use bevy::prelude::*;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

#[derive(Resource)]
pub struct Rng {
    pub inner: Xoshiro256StarStar,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Seed {
    Number(u64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RngPlugin {
    seed: Option<Seed>,
}

impl RngPlugin {
    pub fn new() -> Self {
        Self { seed: None }
    }

    pub fn with_seed(seed: Seed) -> Self {
        Self { seed: Some(seed) }
    }
}

impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        let rng = match &self.seed {
            Some(Seed::Number(num)) => Xoshiro256StarStar::seed_from_u64(*num),
            None => Xoshiro256StarStar::seed_from_u64(0),
        };

        app.insert_resource(Rng { inner: rng });
    }
}
