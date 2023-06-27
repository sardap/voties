use std::{collections::HashSet, ops::Div, time::Duration};

use bevy::prelude::*;
use enum_iterator::Sequence;
use rand::Rng;

use crate::{age, energy, rng, shelter, sim_time::SimTime};

#[derive(Debug, PartialEq, Hash, Copy, Clone, Sequence)]
pub enum DeathReason {
    Starvation,
    OldAge,
    Homeliness,
}

impl DeathReason {
    pub fn get_at_risk_message(&self) -> String {
        match self {
            DeathReason::Starvation => "I am starving".to_string(),
            DeathReason::OldAge => "I am old".to_string(),
            DeathReason::Homeliness => "I am homeless".to_owned(),
        }
    }

    pub fn get_reason_of_death_message(&self) -> String {
        match self {
            DeathReason::Starvation => "Starved".to_string(),
            DeathReason::OldAge => "Olded to death".to_string(),
            DeathReason::Homeliness => "Exposure".to_owned(),
        }
    }
}

impl Eq for DeathReason {}

#[derive(Debug, Component, Default)]
pub struct Mortal {
    pub dead: Option<DeathReason>,
    pub at_risk: HashSet<DeathReason>,
    pub energy_depleted_for: Option<std::time::Duration>,
}

pub fn death_from_exhaustion_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut query: Query<(&mut Mortal, &energy::Energy)>,
) {
    for (mut mortal, energy) in &mut query {
        if energy.current_kcal <= 0.0 {
            if mortal.energy_depleted_for.is_none() {
                mortal.energy_depleted_for = Some(std::time::Duration::ZERO);
            }

            if !mortal.at_risk.contains(&DeathReason::Starvation) {
                mortal.at_risk.insert(DeathReason::Starvation);
            }

            let mut depleted_for = mortal.energy_depleted_for.unwrap();
            depleted_for += sim_time.delta(&time);
            if depleted_for > std::time::Duration::from_secs(30) {
                mortal.dead = Some(DeathReason::Starvation);
            }

            mortal.energy_depleted_for = Some(depleted_for);
        } else {
            mortal.energy_depleted_for = None;
            mortal.at_risk.remove(&DeathReason::Starvation);
        }
    }
}

pub const OLD_AGE_DEATH_THRESHOLD: std::time::Duration = std::time::Duration::from_secs(250);

#[derive(Resource)]
pub struct CheckOldAgeTimer(pub Timer);

pub fn die_of_old_age_system(
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut rng: ResMut<rng::Rng>,
    mut timer: ResMut<CheckOldAgeTimer>,
    mut query: Query<(&mut Mortal, &age::Age)>,
) {
    for _ in 0..timer
        .0
        .tick(sim_time.delta(&time))
        .times_finished_this_tick()
    {
        for (mut mortal, age) in &mut query {
            let death_threshold = OLD_AGE_DEATH_THRESHOLD;

            if age.duration_alive < death_threshold.mul_f64(0.7) {
                continue;
            }

            if !mortal.at_risk.contains(&DeathReason::OldAge) {
                mortal.at_risk.insert(DeathReason::OldAge);
            }

            let max_range = death_threshold + std::time::Duration::from_secs(5);

            let death_number = rng.inner.gen_range(age.duration_alive..max_range);

            if death_number > death_threshold {
                mortal.dead = Some(DeathReason::OldAge);
            }
        }
    }
}

pub fn die_of_homelessness_system(mut query: Query<(&mut Mortal, &shelter::RequiresHouse)>) {
    for (mut mortal, age) in &mut query {
        const HOMELINESS_DEATH_THRESHOLD: Duration = Duration::from_secs(40);

        if age.homeless_for < HOMELINESS_DEATH_THRESHOLD / 2 {
            continue;
        }

        if !mortal.at_risk.contains(&DeathReason::Homeliness) {
            mortal.at_risk.insert(DeathReason::Homeliness);
        }

        if age.homeless_for > HOMELINESS_DEATH_THRESHOLD {
            mortal.dead = Some(DeathReason::Homeliness);
        }
    }
}

pub fn remove_dead_system(mut commands: Commands, query: Query<(Entity, &Mortal)>) {
    for (entity, mortal) in &query {
        if let Some(reason) = &mortal.dead {
            info!("Removing dead entity which died of {:?} from world", reason);
            commands.entity(entity).despawn_recursive();
        }
    }
}
