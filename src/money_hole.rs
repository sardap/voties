use bevy::prelude::*;

use crate::{
    assets, building,
    money::{Money, Treasury},
};

pub const MONEY_HOLE_CAPACITY_MIN: Money = 1000.0;
pub const MONEY_HOLE_CAPACITY_MAX: Money = 3000.0;

#[derive(Component, Clone, Default)]
pub struct MoneyHole {
    pub capacity: Money,
}

#[derive(Component, Clone, Default)]
pub struct MoneyHoleText;

#[derive(Bundle, Clone, Default)]
pub struct MoneyHoleTextBundle {
    pub mint_text: MoneyHoleText,
    #[bundle]
    pub text2d: Text2dBundle,
}

#[derive(Bundle, Clone, Default)]
pub struct MoneyHoleBundle {
    pub farm: MoneyHole,
    pub building_status: building::BuildingStatus,
    #[bundle]
    pub sprite: SpriteBundle,
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: &AssetServer,
    storage_capacity: Money,
    location: Vec2,
) {
    commands
        .spawn(MoneyHoleBundle {
            farm: MoneyHole {
                capacity: storage_capacity,
            },
            sprite: SpriteBundle {
                texture: asset_server.load(crate::assets::DEFAULT_MINT_SPRITE_PATH),
                transform: Transform::from_translation(Vec3::new(location.x, location.y, 0.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(MoneyHoleTextBundle {
                text2d: Text2dBundle {
                    text: Text::from_section(
                        format!("Money Hole Capacity ${}", storage_capacity.round()),
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

pub fn update_treasury_capacity_system(query: Query<&MoneyHole>, mut treasury: ResMut<Treasury>) {
    let new_capacity = query.iter().map(|i| i.capacity).sum::<Money>();

    treasury.change_capacity(new_capacity);
}
