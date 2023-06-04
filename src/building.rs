use bevy::prelude::*;

use crate::hunger::FoodTemplate;

#[derive(Debug, Clone)]
pub enum Building {
    Farm(FoodTemplate),
    ReproductiveZone,
}

impl ToString for Building {
    fn to_string(&self) -> String {
        match self {
            Building::Farm(food_template) => format!("Farm {}", food_template.name),
            Building::ReproductiveZone => "Reproductive Zone".to_string(),
        }
    }
}
