use std::time::Duration;

use bevy::prelude::*;
use bevy_enum_filter::prelude::*;

use crate::{
    assets, building,
    money::{Money, Treasury},
};

pub const MINT_MPS_MIN: Money = 200.0;
pub const MINT_MPS_MAX: Money = 500.0;

#[derive(Component, Clone, Default)]
pub struct Mint {
    pub money_per_cycle: Money,
    pub timer: Timer,
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

pub fn spawn(
    commands: &mut Commands,
    asset_server: &AssetServer,
    money_per_cycle: Money,
    production_time: Duration,
    location: Vec2,
) {
    commands
        .spawn(MintBundle {
            farm: Mint {
                money_per_cycle,
                timer: Timer::from_seconds(production_time.as_secs_f32(), TimerMode::Repeating),
            },
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
                        format!(
                            "Mint Produces ${}/{:.2}s",
                            money_per_cycle.round(),
                            production_time.as_secs_f32()
                        ),
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

pub fn mint_produce_system(
    time: Res<Time>,
    mut treasury: ResMut<Treasury>,
    mut query: Query<&mut Mint, Without<Enum!(building::BuildingStatus::Dilapidated)>>,
) {
    for mut mint in query.iter_mut() {
        if mint.timer.tick(time.delta()).just_finished() {
            // TODO here add plus money symbol
            treasury.add(mint.money_per_cycle);
        }
    }
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
