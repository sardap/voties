use bevy::prelude::*;

use crate::{assets, money::Treasury};

pub fn setup(commands: &mut Commands, asset_server: &AssetServer) {
    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(15.0), Val::Percent(7.0)),
                border: UiRect::all(Val::Px(5.0)),
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(85.0),
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
                                    "Money: $",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                TextSection::new(
                                    "VALUE",
                                    TextStyle {
                                        font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                        font_size: 20.0,
                                        color: Color::DARK_GREEN,
                                    },
                                ),
                            ])
                            .with_alignment(TextAlignment::Center),
                            ..default()
                        })
                        .insert(MoneyAmountText);
                });
        });
}

#[derive(Debug, Component)]
pub struct MoneyAmountText;

pub fn update_election_status_system(
    treasury: Res<Treasury>,
    mut text: Query<&mut Text, With<MoneyAmountText>>,
) {
    let mut text = match text.iter_mut().next() {
        Some(text) => text,
        None => return,
    };

    text.sections[1].value = treasury.money.round().to_string();
}
