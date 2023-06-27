use bevy::prelude::*;

use crate::{assets, elections::election::ElectionClosedEvent};

pub fn setup(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(40.0), Val::Percent(50.0)),
                border: UiRect::all(Val::Px(5.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(30.0),
                    top: Val::Percent(25.0),
                    ..default()
                },
                ..default()
            },
            visibility: Visibility::Hidden,
            background_color: Color::hex("71FFFF").unwrap().into(),
            ..default()
        })
        .insert(ElectionResultRootNode)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::width(Val::Percent(100.0)),
                        justify_content: JustifyContent::Center,
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
                        .spawn(TextBundle {
                            text: Text::from_sections(vec![
                                TextSection::new(
                                    "Election Result\n",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "Main Result",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 15.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "Other Results",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 13.0,
                                        color: Color::BLACK,
                                    },
                                ),
                            ])
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        })
                        .insert(ElectionResultBodyNode);
                });
        });
}

#[derive(Debug, Component)]
pub struct ElectionResultRootNode;

#[derive(Debug, Component)]
pub struct ElectionResultBodyNode;

pub fn update_election_result_system(
    mut election_events: EventReader<ElectionClosedEvent>,
    body_node: Query<Entity, With<ElectionResultBodyNode>>,
    mut text: Query<&mut Text>,
) {
    if election_events.is_empty() {
        return;
    }

    // This feels wrong
    let last_election = election_events.iter().last().unwrap().held_election.clone();
    election_events.clear();

    let mut text = match body_node.iter().next() {
        Some(election) => text.get_mut(election).unwrap(),
        None => return,
    };

    {
        let title = text.sections.get_mut(1).unwrap();
        let winner = last_election.results[0].get_winner();
        title.value = format!(
            "{} {} - Result: {}",
            last_election.election.election_type.to_string(),
            last_election.name,
            winner.to_string()
        );
    }

    {
        let other_results = text.sections.get_mut(2).unwrap();

        let mut str = string_builder::Builder::default();
        str.append("\nOther Results\n");

        for i in 1..last_election.results.len() {
            let result = &last_election.results[i];
            str.append(format!(
                "{} - {}\n",
                result.get_type().to_string(),
                result.get_winner().to_string()
            ));
        }

        other_results.value = str.string().unwrap();
    }
}
