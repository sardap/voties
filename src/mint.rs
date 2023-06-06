use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    assets, building,
    money::{Money, Treasury},
};

#[derive(Component, Clone, Default)]
pub struct Mint {
    pub money_per_second: Money,
}

#[derive(Component, Clone, Default)]
pub struct MintText;

#[derive(Bundle, Clone, Default)]
pub struct MintTextBundle {
    pub mint_text: MintText,
    #[bundle]
    pub text2d: Text2dBundle,
}

#[derive(Bundle, Clone, Default)]
pub struct MintBundle {
    pub farm: Mint,
    pub building_status: building::BuildingStatus,
    #[bundle]
    pub sprite: SpriteBundle,
}

impl MintBundle {
    pub fn spawn(
        commands: &mut Commands,
        asset_server: &AssetServer,
        money_per_second: Money,
        location: Vec2,
    ) {
        commands
            .spawn(Self {
                farm: Mint { money_per_second },
                sprite: SpriteBundle {
                    texture: asset_server.load(crate::assets::DEFAULT_MINT_SPRITE_PATH),
                    transform: Transform::from_translation(Vec3::new(location.x, location.y, 0.0)),
                    ..default()
                },
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(MintTextBundle {
                    text2d: Text2dBundle {
                        text: Text::from_section(
                            format!("Mint Produces ${}/s", money_per_second.round()),
                            TextStyle {
                                font: asset_server.load(assets::DEFAULT_FONT_PATH),
                                font_size: 10.0,
                                color: Color::BLACK,
                            },
                        ),
                        transform: Transform::from_xyz(0.0, 60.0, 10.0),
                        ..default()
                    },
                    ..default()
                });
            });
    }
}

pub fn mint_produce_system(
    time: Res<Time>,
    mut treasury: ResMut<Treasury>,
    query: Query<&Mint, Without<Enum!(building::BuildingStatus::Dilapidated)>>,
) {
    let total_money_per_second: Money = query.iter().map(|mint| mint.money_per_second).sum();
    // TODO here add floating plus text
    treasury.add(total_money_per_second * time.delta_seconds_f64());
}

pub fn mints_have_become_dilapidated_system(
    mut query: Query<
        &mut Sprite,
        (
            With<Mint>,
            Added<Enum!(building::BuildingStatus::Dilapidated)>,
        ),
    >,
) {
    for mut farm in query.iter_mut() {
        farm.color = Color::RED;
    }
}
