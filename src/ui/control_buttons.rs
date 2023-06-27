use bevy::prelude::*;

use crate::assets;

use super::{button, election_result::ElectionResultRootNode};

pub fn setup(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(100.0)),
                border: UiRect::all(Val::Px(5.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    bottom: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            background_color: Color::hex("71FFFF").unwrap().into(),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(100.0)),
                        justify_content: JustifyContent::Start,
                        align_items: AlignItems::Center,
                        padding: UiRect {
                            left: Val::Px(5.0),
                            right: Val::Px(5.0),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: Color::hex("BDFFFF").unwrap().into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(90.0), Val::Px(70.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::all(Val::Px(5.0)),
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            background_color: button::NORMAL_BUTTON.into(),
                            ..default()
                        })
                        .insert(ShowLastElectionButton)
                        .with_children(|parent| {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "Show last election",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 17.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                    },
                                ),
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        left: Val::Px(0.0),
                                        top: Val::Px(0.0),
                                        ..default()
                                    },
                                    align_content: AlignContent::Center,
                                    justify_content: JustifyContent::Center,
                                    max_size: Size::width(Val::Px(80.0)),
                                    ..default()
                                },
                                ..default()
                            });
                        });
                });
        });
}

#[derive(Debug, Component)]
pub struct ShowLastElectionButton;

pub fn show_last_election_button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut root_vis: Query<&mut Visibility, With<ElectionResultRootNode>>,
) {
    let interaction = match interaction_query.iter().next() {
        Some(interaction) => interaction,
        None => return,
    };

    if *interaction != Interaction::Clicked {
        return;
    }

    let mut visibility = match root_vis.iter_mut().next() {
        Some(visibility) => visibility,
        None => return,
    };

    if *visibility == Visibility::Hidden {
        *visibility = Visibility::Visible;
    } else {
        *visibility = Visibility::Hidden;
    }
}
