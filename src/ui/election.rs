use std::time::Duration;

use bevy::prelude::*;

use crate::{assets, elections::election::Election, name};

pub fn setup(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(70.0), Val::Percent(15.0)),
                border: UiRect::all(Val::Px(5.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(15.0),
                    ..default()
                },
                ..default()
            },
            background_color: Color::hex("71FFFF").unwrap().into(),
            ..default()
        })
        .insert(ElectionStatusRootNode)
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
                                    "ELECTION TYPE",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    ":",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "NAME",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                    },
                                ),
                            ])
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        })
                        .insert(ElectionTitleNode);

                    parent
                        .spawn(TextBundle {
                            text: Text::from_sections(vec![
                                TextSection::new(
                                    "\nCloses in: ",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 17.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "VALUE",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 17.0,
                                        color: Color::RED,
                                    },
                                ),
                                TextSection::new(
                                    "\nVotes: ",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 17.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "VALUE",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 17.0,
                                        color: Color::RED,
                                    },
                                ),
                            ])
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        })
                        .insert(ElectionStatusNode);

                    parent
                        .spawn(TextBundle {
                            text: Text::from_sections(vec![
                                TextSection::new(
                                    "Options:\n",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "Values",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 17.0,
                                        color: Color::BLACK,
                                    },
                                ),
                            ])
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        })
                        .insert(ElectionOptionsNode);
                });
        });
}

#[derive(Debug, Component)]
pub struct ElectionStatusRootNode;

#[derive(Debug, Component)]
pub struct ElectionTitleNode;

#[derive(Debug, Component)]
pub struct ElectionStatusNode;

#[derive(Debug, Component)]
pub struct ElectionOptionsNode;

struct ElectionStatus {
    name: String,
    election_type: String,
    open_since: Duration,
    options: Vec<String>,
    votes: usize,
}

impl ToString for ElectionStatus {
    fn to_string(&self) -> String {
        format!(
            "{}\n Type: {}\n Open for: {}\n Options: {}\n Votes: {}",
            self.name,
            self.election_type,
            self.open_since.as_secs(),
            self.options.join(",\n "),
            self.votes
        )
    }
}

pub fn update_election_status_system(
    mut commands: Commands,
    root_vis: Query<(Entity, &Visibility), With<ElectionStatusRootNode>>,
    election_title: Query<Entity, With<ElectionTitleNode>>,
    election_status: Query<Entity, With<ElectionStatusNode>>,
    election_options: Query<Entity, With<ElectionOptionsNode>>,
    mut text: Query<&mut Text>,
    elections: Query<(&Election, &name::Name)>,
) {
    let (root_entity, root_vis) = match root_vis.iter().next() {
        Some(election) => election,
        None => return,
    };

    let mut elections: Vec<_> = elections
        .iter()
        .map(|(election, name)| ElectionStatus {
            name: name.0.to_string(),
            election_type: election.election_type.to_string(),
            open_since: election.time_open,
            options: election.options.iter().map(|x| x.to_string()).collect(),
            votes: election.voted.len(),
        })
        .collect();
    elections.sort_by(|a, b| a.open_since.cmp(&b.open_since));

    if elections.len() == 0 {
        if *root_vis != Visibility::Hidden {
            commands.entity(root_entity).insert(Visibility::Hidden);
        }
        return;
    } else if *root_vis != Visibility::Visible {
        commands.entity(root_entity).insert(Visibility::Visible);
    }

    let election = elections.first().unwrap();

    {
        let mut election_title = match election_title.iter().next() {
            Some(election) => text.get_mut(election).unwrap(),
            None => return,
        };

        let title = election_title.sections.get_mut(0).unwrap();
        title.value = election.election_type.clone();

        let name = election_title.sections.get_mut(2).unwrap();
        name.value = election.name.clone();
    }

    {
        let mut election_status = match election_status.iter().next() {
            Some(election) => text.get_mut(election).unwrap(),
            None => return,
        };

        {
            let closes_in_text = election_status.sections.get_mut(1).unwrap();
            let closes_in = match Duration::from_secs(15).checked_sub(election.open_since) {
                Some(x) => x,
                None => Duration::ZERO,
            };

            closes_in_text.value = closes_in.as_secs().to_string();
        }

        {
            let vote_count = election_status.sections.get_mut(3).unwrap();
            vote_count.value = election.votes.to_string();
        }
    }

    {
        let mut election_options = match election_options.iter().next() {
            Some(election) => text.get_mut(election).unwrap(),
            None => return,
        };

        let options = election_options.sections.get_mut(1).unwrap();
        options.value = election.options.join(",\n ");
    }
}
