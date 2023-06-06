extern crate measurements;
use bevy::{
    app::App,
    core_pipeline::clear_color::ClearColorConfig,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    time::{Timer, TimerMode},
    window::{PresentMode, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_common_assets::toml::TomlAssetPlugin;
use bevy_enum_filter::prelude::*;
use building::BuildingPlots;
use rand::Rng;

mod age;
mod assets;
mod brain;
mod building;
mod collision;
mod death;
mod elections;
mod energy;
mod farm;
mod goals;
mod grave;
mod hunger;
mod info;
mod mint;
mod money;
mod movement;
mod name;
mod people;
mod player;
mod reproduction;
mod rng;
mod sim_setup;
mod stats;
mod text;
mod ui;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let food_collection = FoodCollectionHandle(asset_server.load(assets::FOOD_CONFIG_FILE));
    commands.insert_resource(food_collection);

    commands.spawn(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
        },
        ..default()
    });
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
        100
    };

    App::new()
        .add_enum_filter::<building::BuildingStatus>()
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
        .add_plugin(rng::RngPlugin::with_seed(rng::Seed::Number(seed)))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(stats::StatsPlugin)
        .add_state::<AppState>()
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
            ui::election::update_election_status_system.in_set(UiSet::Normal),
            ui::button::button_color_system.in_set(UiSet::Normal),
            ui::resources::building_button_system.in_set(UiSet::Normal),
            ui::money::update_election_status_system.in_set(UiSet::Normal),
        ))
        .add_systems((
            hunger::drain_stomach_system.in_set(LifeSet::World),
            energy::drain_energy_system.in_set(LifeSet::World),
            brain::decide_system.in_set(LifeSet::Decide),
            people::update_info_text.in_set(LifeSet::World),
            goals::step_hunger_goal_system.in_set(LifeSet::Goal),
            goals::step_reproduce_goal_system.in_set(LifeSet::Goal),
            goals::step_wander_goal_system.in_set(LifeSet::Goal),
            age::age_up_system.in_set(LifeSet::World),
            farm::farms_make_food_system.in_set(LifeSet::World),
            people::give_birth_system.in_set(LifeSet::World),
            death::death_from_exhaustion_system.in_set(LifeSet::Mortal),
            death::die_of_old_age_system.in_set(LifeSet::Mortal),
            elections::election::start_election_system.in_set(LifeSet::World),
            goals::vote_goal_system.in_set(LifeSet::Goal),
            elections::election::close_elections_system.in_set(LifeSet::World),
        ))
        .add_systems((
            farm::update_farm_tint_system.in_set(LifeSet::World),
            mint::mints_have_become_dilapidated_system.in_set(LifeSet::World),
            mint::mint_produce_system.in_set(LifeSet::World),
            money::upkeep_cost_system.in_set(LifeSet::World),
            building::change_building_status_system.in_set(LifeSet::World),
        ))
        .add_systems(
            (people::create_grave_system, death::remove_dead_system)
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
        .configure_set(UiSet::Normal.run_if(in_state(AppState::Running)))
        .run();
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Loading,
    SettingUpWorld,
    SettingUpUi,
    Running,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum LifeSet {
    World,
    Decide,
    Goal,
    Mortal,
    MortalResponse,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum UiSet {
    Normal,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PhysicsSet {
    Movement,
    CollisionDetection,
}

#[derive(Resource)]
struct FoodCollectionHandle(Handle<hunger::FoodCollection>);
