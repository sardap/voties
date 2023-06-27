use std::time::Duration;

use bevy::{ecs::world, prelude::*, utils::HashMap};
use num_traits::ToPrimitive;

use crate::{
    buildings::{building, house::House},
    death::DeathReason,
    elections::voter::Voter,
    grave::Grave,
    money,
    sim_time::SimTime,
};

// Two election cycles
const HISTORY_LENGTH: usize = 80;

#[derive(Debug, Clone, Copy)]
pub struct Stat<T> {
    history: [Option<T>; HISTORY_LENGTH],
}

impl<T: std::marker::Copy> Default for Stat<T> {
    fn default() -> Self {
        Self {
            history: [None; HISTORY_LENGTH],
        }
    }
}

impl<
        T: num_traits::ToPrimitive + std::cmp::PartialOrd + std::default::Default + std::marker::Copy,
    > Stat<T>
{
    pub fn push(&mut self, value: T) {
        self.history.rotate_right(1);
        self.history[0] = Some(value);
    }

    pub fn max(&self) -> T {
        let mut max: T = T::default();

        for value in self.history.iter() {
            match value {
                Some(val) => {
                    if *val > max {
                        max = *val;
                    }
                }
                None => {}
            }
        }

        max
    }

    pub fn min(&self) -> T {
        let mut min: T = T::default();

        for value in self.history.iter() {
            match value {
                Some(val) => {
                    if *val < min {
                        min = *val;
                    }
                }
                None => {}
            }
        }

        min
    }

    pub fn average(&self) -> f64 {
        let mut sum: f64 = 0.0;

        for value in self.history.iter() {
            match value {
                Some(val) => sum += val.to_f64().unwrap(),
                None => {}
            }
        }

        let length: f64 = self.history.len().to_f64().unwrap();

        sum / length
    }

    pub fn median(&self) -> T {
        let mut history: Vec<_> = self.history.iter().filter_map(|i| i.as_ref()).collect();
        if history.len() == 0 {
            return T::default();
        }

        history.sort_by(|a, b| a.partial_cmp(b).unwrap());

        *history[self.history.len() / 2]
    }

    pub fn latest(&self) -> T {
        match self.history[0] {
            Some(value) => value,
            None => T::default(),
        }
    }
}

#[derive(Debug)]
pub struct Count<T: std::hash::Hash> {
    map: HashMap<T, usize>,
}

impl<T: std::hash::Hash> Default for Count<T> {
    fn default() -> Self {
        Self {
            map: HashMap::default(),
        }
    }
}

impl<T: std::hash::Hash + std::cmp::Eq + std::clone::Clone> Count<T> {
    pub fn update(&mut self, value: &T, count: usize) {
        self.map.insert(value.clone(), count);
    }

    pub fn count(&self) -> usize {
        self.map.iter().map(|(_, count)| *count).sum()
    }

    pub fn get(&self, value: &T) -> usize {
        match self.map.get(value) {
            Some(count) => *count,
            None => 0,
        }
    }

    pub fn min(&self) -> usize {
        *self.map.iter().min_by_key(|(_, count)| *count).unwrap().1
    }

    pub fn max(&self) -> usize {
        *self.map.iter().max_by_key(|(_, count)| *count).unwrap().1
    }

    pub fn sum(&self) -> usize {
        self.map.iter().map(|(_, count)| *count).sum()
    }
}

#[derive(Debug, Resource)]
pub struct WorldStats {
    timer: Timer,
    pub money: Stat<money::Money>,
    pub hole_filled_capacity: Stat<f64>,
    pub buildings: Count<building::BuildingStatus>,
    pub houses_filled: Stat<f32>,
    pub deaths: Count<DeathReason>,
    pub population: Stat<usize>,
}

impl WorldStats {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
            money: Stat::default(),
            hole_filled_capacity: Stat::default(),
            buildings: Count::default(),
            houses_filled: Stat::default(),
            deaths: Count::default(),
            population: Stat::default(),
        }
    }
}

pub fn world_stats_update_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut world_stats: ResMut<WorldStats>,
    treasury: Res<money::Treasury>,
    buildings: Query<&building::BuildingStatus>,
    houses: Query<&House>,
    graves: Query<&Grave>,
    voties: Query<&Voter>,
) {
    if !world_stats
        .timer
        .tick(sim_time.delta(&time))
        .just_finished()
    {
        return;
    }

    world_stats.money.push(treasury.money);
    world_stats
        .hole_filled_capacity
        .push(treasury.money / treasury.capacity);

    // Housing states
    let mut houses_capacity = 0;
    let mut houses_occupants = 0;
    for house in houses.iter() {
        houses_capacity += house.dwellings;
        houses_occupants += house.occupants_count();
    }
    world_stats
        .houses_filled
        .push(houses_occupants as f32 / houses_capacity as f32);

    // Graves
    let grave_cutoff = match sim_time.elapsed().checked_sub(Duration::from_secs(60)) {
        Some(val) => val,
        None => Duration::ZERO,
    };
    for death_reason in enum_iterator::all::<DeathReason>() {
        // SLOW POINT
        world_stats.deaths.update(
            &death_reason,
            graves
                .iter()
                .filter(|g| g.created > grave_cutoff && g.died_of == death_reason)
                .count(),
        );
    }

    // Buildings
    for building in enum_iterator::all::<building::BuildingStatus>() {
        // SLOW POINT
        world_stats.buildings.update(
            &building,
            buildings.iter().filter(|b| **b == building).count(),
        );
    }

    world_stats.population.push(voties.iter().count());
}
