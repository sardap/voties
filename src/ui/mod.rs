pub mod button;
pub mod camera;
mod control_buttons;
pub mod election_result;
pub mod election_status;
pub mod info_text;
pub mod money;

use bevy::prelude::*;

use crate::AppState;

#[derive(Debug, Component)]
pub struct UiRoot;

pub fn setup(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    let root = commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::width(Val::Percent(100.0)),
                ..default()
            },
            ..default()
        })
        .insert(UiRoot)
        .id();

    election_status::setup(&mut commands, &asset_server);
    election_result::setup(&mut commands, &asset_server);
    money::setup(&mut commands, &asset_server);
    control_buttons::setup(&mut commands, &asset_server);

    state.set(AppState::Running);
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum UiSet {
    Normal,
}

#[derive(Debug)]
pub struct VotiesUiPlugin;

impl Plugin for VotiesUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(camera::setup);

        app.add_systems((
            election_status::update_election_status_system.in_set(UiSet::Normal),
            election_result::update_election_result_system.in_set(UiSet::Normal),
            button::button_color_system.in_set(UiSet::Normal),
            money::update_election_status_system.in_set(UiSet::Normal),
            control_buttons::show_last_election_button_system.in_set(UiSet::Normal),
        ))
        .configure_set(UiSet::Normal.run_if(in_state(AppState::Running)));
    }
}
