pub mod button;
pub mod election;
pub mod resources;

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

    election::setup(&mut commands, &asset_server);
    resources::setup(root, &mut commands, &asset_server);

    state.set(AppState::Running);
}
