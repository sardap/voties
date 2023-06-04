use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Name(pub String);

#[derive(Resource)]
pub struct NameGenerator {
    top_number: u64,
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self { top_number: 1 }
    }
}

impl NameGenerator {
    pub fn generate(&mut self) -> String {
        let name = format!("#{}", self.top_number);
        self.top_number += 1;
        name
    }
}
