use bevy::prelude::*;

pub type Money = f64;

#[derive(Debug, Resource)]
pub struct Treasury {
    pub money: Money,
    pub capacity: Money,
}

impl Treasury {
    pub fn new() -> Self {
        Self {
            money: 0.0,
            capacity: 0.0,
        }
    }

    pub fn change_capacity(&mut self, new_capacity: Money) {
        self.capacity = new_capacity;

        if self.money > self.capacity {
            self.money = self.capacity;
        }
    }

    pub fn spend(&mut self, amount: Money) -> bool {
        if self.money >= amount {
            self.money -= amount;
            true
        } else {
            false
        }
    }

    pub fn add(&mut self, amount: Money) {
        self.money += amount;
    }

    pub fn get_capacity(&self) -> Money {
        self.capacity
    }

    pub fn get_filled_percentage(&self) -> f32 {
        (self.money / self.capacity) as f32
    }
}
