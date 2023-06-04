use bevy::prelude::*;

use crate::{
    assets, building::Building, farm::create_farm, reproduction::ReproductiveZoneBundle, rng,
};

use super::button;

#[derive(Debug, Bundle)]
pub struct BuildingButtonBundle {
    pub building: BuildingButton,
    #[bundle]
    pub button: ButtonBundle,
}

pub fn setup(parent: Entity, commands: &mut Commands, asset_server: &AssetServer) {
    commands.entity(parent).with_children(|parent| {
        parent
            .spawn(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(70.0), Val::Percent(15.0)),
                    position: UiRect {
                        top: Val::Percent(85.0),
                        left: Val::Px(0.0),
                        ..default()
                    },
                    border: UiRect::all(Val::Px(5.0)),
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
                            ..default()
                        },
                        background_color: Color::hex("BDFFFF").unwrap().into(),
                        ..default()
                    })
                    .insert(BuildingButtonNode);
            });
    });
}

pub fn crete_building_button(
    commands: &mut Commands,
    building_button_node: Entity,
    asset_server: &AssetServer,
    building: Building,
) {
    let name = building.to_string();

    let button = commands
        .spawn(BuildingButtonBundle {
            building: BuildingButton { target: building },
            button: ButtonBundle {
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
            },
        })
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    name,
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
                    max_size: Size::width(Val::Px(80.0)),
                    ..default()
                },
                ..default()
            });
        })
        .id();

    commands.entity(building_button_node).add_child(button);
}

#[derive(Debug, Component, Default)]
pub struct BuildingButtonNode;

#[derive(Debug, Component)]
pub struct BuildingButton {
    target: Building,
}

pub fn building_button_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<rng::Rng>,
    mut interaction_query: Query<
        (Entity, &Interaction, &BuildingButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (entity, interaction, building) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                match building.target.clone() {
                    Building::Farm(produces) => {
                        create_farm(
                            &mut commands,
                            &asset_server,
                            produces,
                            Vec3::new(50.0, 80.0, 0.0),
                            &mut rng.inner,
                        );
                    }
                    Building::ReproductiveZone => {
                        commands.spawn(ReproductiveZoneBundle::new(
                            &asset_server,
                            Vec2::new(50.0, 200.0),
                        ));
                    }
                }

                commands.entity(entity).despawn_recursive();
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
