use std::{collections::HashSet, time::Duration};

use bevy::prelude::*;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;

use crate::{
    death,
    elections::voting_center::VotingCenterBundle,
    farm::{self},
    hunger, name,
    people::{self, create_person},
    reproduction::ReproductiveZoneBundle,
    rng, AppState,
};

pub fn setting_up_world(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut rng: ResMut<rng::Rng>,
    food_collection: Res<hunger::FoodCollection>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut name_gen: ResMut<name::NameGenerator>,
) {
    create_starting_farms(
        &mut commands,
        &asset_server,
        &mut rng.inner,
        &food_collection,
    );

    create_reproductive_zone(&mut commands, &asset_server);

    create_starting_people(
        &mut commands,
        &time,
        &asset_server,
        &mut rng.inner,
        &mut name_gen,
    );

    create_voting_center(&mut commands, &asset_server);

    state.set(AppState::SettingUpUi);
}

fn create_starting_farms(
    commands: &mut Commands,
    asset_server: &AssetServer,
    rng: &mut impl rand::Rng,
    food_collection: &hunger::FoodCollection,
) {
    for i in 0..2 {
        let produces = food_collection.foods.choose(rng).unwrap().clone();

        farm::create_farm(
            commands,
            asset_server,
            produces,
            Vec3::new(i as f32 * 150.0, 120.0, 0.0),
            rng,
        );
    }
}

fn create_reproductive_zone(commands: &mut Commands, asset_server: &AssetServer) {
    for i in 0..1 {
        commands.spawn(ReproductiveZoneBundle::new(
            &asset_server,
            Vec2::new((i + 1) as f32 * -150.0, 120.0),
        ));
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
        let mut food_groups = HashSet::new();
        people::fill_wont_eat(food_group_count, &mut food_groups, rng);
        let food_groups = food_groups.into_iter().collect::<Vec<_>>();

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
            &food_groups,
        );
    }
}

fn create_voting_center(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn(VotingCenterBundle::new(
        asset_server,
        Vec3::new(0.0, 0.0, 0.0),
    ));
}
