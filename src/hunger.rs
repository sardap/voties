use bevy::reflect::TypeUuid;
use bevy::{prelude::*, time::Time};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::{collections::HashSet, ops::Sub};
use strum_macros::EnumIter;

use crate::money::Money;

#[derive(Component)]
pub struct Stomach {
    pub max_size_ml: f64,
    pub filled_ml: f64,
}

impl Default for Stomach {
    fn default() -> Self {
        Self {
            max_size_ml: 1000.0,
            filled_ml: 500.0,
        }
    }
}

impl Stomach {
    pub fn new(max_size: measurements::Volume, filled: measurements::Volume) -> Self {
        Self {
            max_size_ml: max_size.as_milliliters(),
            filled_ml: filled.as_milliliters(),
        }
    }

    pub fn can_eat(&self, food: &Food) -> bool {
        self.filled_ml + food.ml <= self.max_size_ml
    }

    pub fn percent_filled(&self) -> f64 {
        self.filled_ml / self.max_size_ml
    }
}

#[derive(EnumIter, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum FoodGroup {
    Fruit,
    Vegetable,
    Grain,
    Meat,
    Dairy,
    Fat,
    Sugar,
}

impl Eq for FoodGroup {}

impl FoodGroup {
    pub fn upkeep_multiplier(&self) -> f64 {
        match self {
            FoodGroup::Fruit => 0.4,
            FoodGroup::Vegetable => 0.6,
            FoodGroup::Grain => 0.1,
            FoodGroup::Meat => 2.0,
            FoodGroup::Dairy => 0.8,
            FoodGroup::Fat => 0.8,
            FoodGroup::Sugar => 0.5,
        }
    }
}

#[derive(Debug, Component, Clone, Default, PartialEq)]
pub struct Food {
    pub kcal: f64,
    pub ml: f64,
    pub groups: Vec<FoodGroup>,
}

impl Food {
    pub fn how_long_to_eat(&self) -> std::time::Duration {
        let seconds = self.ml / 500.0;
        std::time::Duration::from_secs_f64(seconds)
    }
}

#[derive(Debug, Component)]
pub struct FoodPreferences {
    pub wont_eat: HashSet<FoodGroup>,
}

impl FoodPreferences {
    pub fn new(wont_eat: &[FoodGroup]) -> Self {
        Self {
            wont_eat: wont_eat.iter().map(|i| *i).collect(),
        }
    }

    pub fn will_eat(&self, food: &Food) -> bool {
        !food.groups.iter().any(|g| self.wont_eat.contains(g))
    }
}

pub fn drain_stomach_system(time: Res<Time>, mut query: Query<&mut Stomach>) {
    for mut stomach in &mut query {
        let new_value = stomach.filled_ml.sub(
            measurements::Volume::from_milliliters(50.0).as_milliliters()
                * time.delta_seconds() as f64,
        );
        if new_value > 0.0 {
            stomach.filled_ml = new_value;
        } else {
            stomach.filled_ml = 0.0;
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
pub struct FoodTemplate {
    pub name: String,
    pub kcal: f64,
    pub ml: f64,
    pub groups: Vec<FoodGroup>,
    pub difficulty: u32,
}

impl Eq for FoodTemplate {}

impl Hash for FoodTemplate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl FoodTemplate {
    pub fn get_food(&self) -> Food {
        Food {
            kcal: self.kcal * 5.0,
            ml: self.ml * 5.0,
            groups: self.groups.clone(),
        }
    }

    pub fn upkeep_cost(&self) -> Money {
        let multiplier = self
            .groups
            .iter()
            .map(|i| i.upkeep_multiplier())
            .sum::<f64>();

        self.ml * multiplier
    }
}

impl Into<Food> for FoodTemplate {
    fn into(self) -> Food {
        self.get_food()
    }
}

#[derive(Deserialize, TypeUuid, Resource)]
#[uuid = "922da48b-8fd1-4fe4-8451-548dd8e91627"]
pub struct FoodCollection {
    pub foods: Vec<FoodTemplate>,
}
