#![feature(const_fn_floating_point_arithmetic)]
#![feature(generic_const_exprs)]
extern crate measurements;
use bevy::{
    app::App,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    time::{Timer, TimerMode},
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_common_assets::toml::TomlAssetPlugin;
use bevy_enum_filter::prelude::*;
use buildings::building::BuildingPlots;
use rand::Rng;
use sets::{AppState, LifeSet};
use world_stats::WorldStats;

mod age;
mod assets;
mod brain;
mod buildings;
mod collision;
mod death;
mod elections;
mod energy;
mod goals;
mod grave;
mod hunger;
mod info;
mod input;
mod money;
mod movement;
mod name;
mod people;
mod player;
mod reproduction;
mod rng;
mod sets;
mod shelter;
mod sim_setup;
mod sim_time;
mod stats;
mod text;
mod ui;
mod upkeep;
mod world_stats;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let food_collection = FoodCollectionHandle(asset_server.load(assets::FOOD_CONFIG_FILE));
    commands.insert_resource(food_collection);
}

fn loading_world_assets(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    food_collection_handle: Res<FoodCollectionHandle>,
    mut food_collection_asset: ResMut<Assets<hunger::FoodCollection>>,
) {
    if let Some(food_collection) = food_collection_asset.remove(food_collection_handle.0.id()) {
        commands.insert_resource(food_collection);
        state.set(AppState::SettingUpWorld);
    }
}

fn main() {
    let seed: u64 = if cfg!(wasm32) {
        rand::thread_rng().gen()
    } else {
        101
    };

    App::new()
        .add_enum_filter::<buildings::building::BuildingStatus>()
        .add_enum_filter::<goals::Goals>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "VotingVotingVoting".into(),
                resolution: (800., 600.).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugin(TomlAssetPlugin::<hunger::FoodCollection>::new(&[
            "food_collection.toml",
        ]))
        .insert_resource(death::CheckOldAgeTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .insert_resource(collision::CollisionTimer(Timer::from_seconds(
            0.05,
            TimerMode::Repeating,
        )))
        .insert_resource(elections::election::ElectionTimer(Timer::from_seconds(
            20.0,
            TimerMode::Repeating,
        )))
        .insert_resource(name::NameGenerator::default())
        .insert_resource(elections::election::ElectionHistory::default())
        .insert_resource(BuildingPlots::new())
        .insert_resource(money::Treasury::new())
        .insert_resource(WorldStats::new())
        .add_plugin(rng::RngPlugin::with_seed(rng::Seed::Number(seed)))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(stats::StatsPlugin)
        .add_state::<AppState>()
        .add_event::<elections::election::ElectionClosedEvent>()
        .add_plugin(ui::VotiesUiPlugin)
        .add_plugin(goals::GoalsPlugin)
        .add_startup_system(setup)
        .add_system(loading_world_assets.run_if(in_state(AppState::Loading)))
        .add_system(sim_setup::setting_up_world.run_if(in_state(AppState::SettingUpWorld)))
        .add_system(ui::setup.run_if(in_state(AppState::SettingUpUi)))
        .add_systems(
            (movement::go_to_target, movement::apply_velcoity_system)
                .chain()
                .in_set(PhysicsSet::Movement),
        )
        .add_system(collision::collision_detection_system.in_set(PhysicsSet::CollisionDetection))
        .configure_set(PhysicsSet::Movement.before(PhysicsSet::CollisionDetection))
        .configure_set(PhysicsSet::Movement.run_if(in_state(AppState::Running)))
        .configure_set(PhysicsSet::CollisionDetection.run_if(in_state(AppState::Running)))
        .add_systems((
            hunger::drain_stomach_system.in_set(LifeSet::World),
            energy::drain_energy_system.in_set(LifeSet::World),
            brain::decide_system.in_set(LifeSet::Decide),
            people::update_info_text.in_set(LifeSet::World),
            age::age_up_system.in_set(LifeSet::World),
            buildings::farm::farms_make_food_system.in_set(LifeSet::World),
            people::give_birth_system.in_set(LifeSet::World),
            death::death_from_exhaustion_system.in_set(LifeSet::Mortal),
            death::die_of_old_age_system.in_set(LifeSet::Mortal),
            elections::election::start_election_system.in_set(LifeSet::World),
            elections::election::close_elections_system.in_set(LifeSet::World),
        ))
        .add_systems((
            buildings::building::update_building_tint_system.in_set(LifeSet::World),
            buildings::mint::mints_have_become_dilapidated_system.in_set(LifeSet::World),
            buildings::mint::mint_produce_system.in_set(LifeSet::World),
            upkeep::upkeep_cost_system.in_set(LifeSet::World),
            buildings::building::change_building_status_system.in_set(LifeSet::World),
            buildings::money_hole::update_treasury_capacity_system.in_set(LifeSet::World),
            world_stats::world_stats_update_system.in_set(LifeSet::World),
            buildings::house::update_house_text_system.in_set(LifeSet::World),
            shelter::tick_homeless_system.in_set(LifeSet::World),
            death::die_of_homelessness_system.in_set(LifeSet::Mortal),
            buildings::house::clear_dead_from_house_system.in_set(LifeSet::World),
            buildings::house::empty_dilapidated_house_system.in_set(LifeSet::World),
            reproduction::reproductive_timer_tick_system.in_set(LifeSet::World),
            sim_time::tick_sim_time_system.in_set(LifeSet::World),
        ))
        .add_systems((
            input::player_input_camera_system.run_if(in_state(AppState::Running)),
            input::player_input_sim_time_system.run_if(in_state(AppState::Running)),
        ))
        .add_systems(
            (grave::create_grave_system, death::remove_dead_system)
                .chain()
                .in_set(LifeSet::MortalResponse),
        )
        .configure_set(LifeSet::World.before(LifeSet::Decide))
        .configure_set(LifeSet::Decide.before(LifeSet::Goal))
        .configure_set(LifeSet::Goal.before(LifeSet::Mortal))
        .configure_set(LifeSet::Mortal.before(LifeSet::MortalResponse))
        .configure_set(LifeSet::World.run_if(in_state(AppState::Running)))
        .configure_set(LifeSet::Decide.run_if(in_state(AppState::Running)))
        .configure_set(LifeSet::Goal.run_if(in_state(AppState::Running)))
        .configure_set(LifeSet::Mortal.run_if(in_state(AppState::Running)))
        .configure_set(LifeSet::MortalResponse.run_if(in_state(AppState::Running)))
        .run();
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    Movement,
    CollisionDetection,
}

#[derive(Resource)]
struct FoodCollectionHandle(Handle<hunger::FoodCollection>);
