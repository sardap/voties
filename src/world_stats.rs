use std::time::Duration;

use bevy::{prelude::*, utils::HashMap};
use num_traits::ToPrimitive;

use crate::{building, money};

// Two election cycles
const HISTORY_LENGTH: usize = 80;

#[derive(Debug)]
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

        *history[HISTORY_LENGTH / 2]
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
}

#[derive(Debug, Resource)]
pub struct WorldStats {
    timer: Timer,
    pub money: Stat<money::Money>,
    pub hole_filled_capacity: Stat<f64>,
    pub buildings: Count<building::BuildingStatus>,
}

impl WorldStats {
    pub fn new() -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
            money: Stat::default(),
            hole_filled_capacity: Stat::default(),
            buildings: Count::default(),
        }
    }
}

pub fn world_stats_update_system(
    time: Res<Time>,
    mut world_stats: ResMut<WorldStats>,
    treasury: Res<money::Treasury>,
    buildings: Query<&building::BuildingStatus>,
) {
    if !world_stats.timer.tick(time.delta()).just_finished() {
        return;
    }

    world_stats.money.push(treasury.money);
    world_stats
        .hole_filled_capacity
        .push(treasury.money / treasury.capacity);

    for building in enum_iterator::all::<building::BuildingStatus>() {
        world_stats.buildings.update(
            &building,
            buildings.iter().filter(|b| **b == building).count(),
        );
    }
}
