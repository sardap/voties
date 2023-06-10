use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;

use crate::{
    building::{Building, BuildingPlots},
    death,
    elections::voting_center::VotingCenterBundle,
    hunger, mint, money_hole, name,
    people::{self, create_person},
    rng,
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

    create_starting_farms(
        &mut commands,
        &asset_server,
        &mut plots,
        &mut rng.inner,
        &food_collection,
    );

    create_reproductive_zone(&mut commands, &asset_server, &mut plots, &mut rng.inner);

    create_starting_people(
        &mut commands,
        &time,
        &asset_server,
        &mut rng.inner,
        &mut name_gen,
    );

    create_voting_center(&mut commands, &asset_server);

    crete_starting_mints(&mut commands, &asset_server, &mut plots, &mut rng.inner);

    create_starting_money_hole(&mut commands, &asset_server, &mut plots, &mut rng.inner);

    state.set(AppState::SettingUpUi);
}

fn create_starting_farms(
    commands: &mut Commands,
    asset_server: &AssetServer,
    plots: &mut BuildingPlots,
    rng: &mut impl rand::Rng,
    food_collection: &hunger::FoodCollection,
) {
    for _ in 0..2 {
        let produces = food_collection.foods.choose(rng).unwrap().clone();

        Building::Farm(produces.clone()).build(commands, asset_server, plots, rng);
    }
}

fn create_reproductive_zone(
    commands: &mut Commands,
    asset_server: &AssetServer,
    plots: &mut BuildingPlots,
    rng: &mut impl rand::Rng,
) {
    for _ in 0..1 {
        Building::ReproductiveZone.build(commands, asset_server, plots, rng);
    }
}

const PEOPLE_COUNT: usize = 10;

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

fn create_voting_center(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn(VotingCenterBundle::new(
        asset_server,
        Vec3::new(0.0, 0.0, 0.0),
    ));
}

fn crete_starting_mints(
    commands: &mut Commands,
    asset_server: &AssetServer,
    plots: &mut BuildingPlots,
    rng: &mut impl rand::Rng,
) {
    let amount = rng.gen_range(mint::MINT_MPS_MIN..mint::MINT_MPS_MAX);

    for _ in 0..1 {
        mint::spawn(
            commands,
            asset_server,
            amount,
            Duration::from_secs(1),
            plots.next(),
        );
    }
}

fn create_starting_money_hole(
    commands: &mut Commands,
    asset_server: &AssetServer,
    plots: &mut BuildingPlots,
    rng: &mut impl rand::Rng,
) {
    let storage_capacity =
        rng.gen_range(money_hole::MONEY_HOLE_CAPACITY_MIN..money_hole::MONEY_HOLE_CAPACITY_MAX);

    for _ in 0..1 {
        money_hole::spawn(commands, asset_server, storage_capacity, plots.next());
    }
}
