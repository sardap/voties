use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;
use lazy_static::lazy_static;
use rand::{seq::SliceRandom, Rng};

use crate::{
    buildings::building::{Building, BuildingPlots},
    buildings::{
        building,
        house::{self, MAX_DWELLINGS, MIN_DWELLINGS},
        mint, money_hole,
        voting_center::VotingCenterBundle,
    },
    death, hunger, name,
    people::{self, create_person},
    rng, sim_time,
    upkeep::setup_upkeep,
    AppState,
};

pub fn setting_up_world(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut rng: ResMut<rng::Rng>,
    mut plots: ResMut<BuildingPlots>,
    food_collection: Res<hunger::FoodCollection>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut name_gen: ResMut<name::NameGenerator>,
) {
    setup_upkeep(&mut commands);

    sim_time::setup(&mut commands);

    create_starting_people(
        &mut commands,
        &time,
        &asset_server,
        &mut rng.inner,
        &mut name_gen,
    );

    let mut buildings_to_create: Vec<Building> = vec![];

    const FARMS_COUNT: usize = PEOPLE_COUNT / 15 + 1;
    const HOUSES_COUNT: usize = PEOPLE_COUNT / (MAX_DWELLINGS - MIN_DWELLINGS) as usize + 1;
    const REPRODUCTIVE_ZONE_COUNT: usize = PEOPLE_COUNT / 100 + 1;
    const MINTS_COUNT: usize =
        FARMS_COUNT / 3 + HOUSES_COUNT / 10 + REPRODUCTIVE_ZONE_COUNT / 10 + 1;
    const MONEY_HOLES_COUNT: usize = MINTS_COUNT / 10 + 1;

    // Add Farms
    for _ in 0..FARMS_COUNT {
        let food = food_collection
            .foods
            .choose(&mut rng.inner)
            .unwrap()
            .clone();
        buildings_to_create.push(Building::Farm(food));
    }

    // Add housing
    for _ in 0..HOUSES_COUNT {
        let dwellings = rng.inner.gen_range(MIN_DWELLINGS..MAX_DWELLINGS);
        buildings_to_create.push(Building::House(dwellings));
    }

    // Add Reproductive Zones
    for _ in 0..REPRODUCTIVE_ZONE_COUNT {
        buildings_to_create.push(Building::ReproductiveZone);
    }

    // Add Mints
    for _ in 0..MINTS_COUNT {
        buildings_to_create.push(Building::Mint);
    }

    // Add Money Holes
    for _ in 0..MONEY_HOLES_COUNT {
        buildings_to_create.push(Building::MoneyHole);
    }

    buildings_to_create.shuffle(&mut rng.inner);

    // Add voting centers
    let voting_center_count = buildings_to_create.len() / 50 + 1;

    for i in 0..voting_center_count {
        buildings_to_create.insert(i * 50, Building::VotingCenter);
    }

    for building in buildings_to_create {
        building.build(&mut commands, &asset_server, &mut plots, &mut rng.inner);
    }

    state.set(AppState::SettingUpUi);
}

const PEOPLE_COUNT: usize = 100;

lazy_static! {
    static ref MIN_STOMACH_SIZE_ML: measurements::Volume = measurements::Volume::from_liters(2.0);
    static ref MAX_STOMACH_SIZE_ML: measurements::Volume = measurements::Volume::from_liters(3.0);
}

const MAX_SPEED: f32 = 200.0;
const MIN_SPEED: f32 = 50.0;

fn create_starting_people(
    commands: &mut Commands,
    time: &Time,
    asset_server: &AssetServer,
    rng: &mut impl rand::Rng,
    name_gen: &mut name::NameGenerator,
) {
    for _ in 0..PEOPLE_COUNT {
        let age =
            Duration::from_secs(rng.gen_range(0..(death::OLD_AGE_DEATH_THRESHOLD.as_secs() / 5)));
        let speed = rng.gen_range(MIN_SPEED..MAX_SPEED);

        let food_group_count = people::wont_eat_count(rng);
        let mut wont_eat = HashSet::new();
        people::fill_wont_eat(food_group_count, &mut wont_eat, rng);
        let prefer_eat_groups = people::create_prefer_eat(rng, &wont_eat, None);
        let wont_eat_food_groups = wont_eat.into_iter().collect::<Vec<_>>();

        let spawn_location = Vec3::new(
            rng.gen_range(-300.0..300.0),
            rng.gen_range(-100.0..100.0),
            10.0,
        );

        create_person(
            commands,
            time,
            asset_server,
            rng,
            name_gen.generate().as_str(),
            spawn_location,
            (MIN_STOMACH_SIZE_ML.clone(), MAX_STOMACH_SIZE_ML.clone()),
            speed,
            age,
            &wont_eat_food_groups,
            &prefer_eat_groups,
        );
    }
}
