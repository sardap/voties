use bevy::{prelude::*, utils::HashSet};
use bevy_enum_filter::prelude::*;

use crate::{assets, buildings, collision, money::Money, shelter, upkeep};

use super::building;

#[derive(Component, Clone, Default)]
pub struct House {
    pub occupied: HashSet<Entity>,
    pub dwellings: i32,
}

pub enum AddOccupantError {
    HouseFull,
}

impl House {
    pub fn is_full(&self) -> bool {
        self.occupied.len() >= self.dwellings as usize
    }

    pub fn add_occupant(&mut self, occupant: Entity) -> Result<(), AddOccupantError> {
        if self.is_full() {
            return Err(AddOccupantError::HouseFull);
        }

        self.occupied.insert(occupant);
        Ok(())
    }

    pub fn occupants_count(&self) -> usize {
        self.occupied.len()
    }
}

#[derive(Component, Clone, Default)]
pub struct HouseText;

#[derive(Bundle, Clone, Default)]
pub struct HouseBundle {
    pub house: House,
    pub building_status: building::BuildingStatus,
    pub upkeep: upkeep::UpkeepCost,
    pub collider: collision::Collider,
    #[bundle]
    pub sprite: SpriteBundle,
}

pub const MIN_DWELLINGS: i32 = 3;
pub const MAX_DWELLINGS: i32 = 10;

pub fn spawn(commands: &mut Commands, asset_server: &AssetServer, dwellings: i32, location: Vec2) {
    let upkeep_cost = (dwellings as Money * 100.0) / 60.0;

    let id = commands
        .spawn(HouseBundle {
            house: House {
                dwellings,
                occupied: HashSet::default(),
            },
            upkeep: upkeep::UpkeepCost::new(upkeep_cost),
            sprite: SpriteBundle {
                texture: asset_server.load(crate::assets::DEFAULT_HOUSE_SPRITE_PATH),
                transform: Transform::from_translation(Vec3::new(location.x, location.y, 0.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Text2dBundle {
                    text: Text::from_section(
                        "",
                        TextStyle {
                            font: asset_server.load(assets::DEFAULT_FONT_PATH),
                            font_size: 10.0,
                            color: Color::BLACK,
                        },
                    ),
                    transform: Transform::from_xyz(0.0, 60.0, 10.0),
                    ..default()
                })
                .insert(HouseText);
        });
}

fn update_house_text(house: &House, upkeep: &upkeep::UpkeepCost, text: &mut Text) {
    text.sections[0].value = format!(
        "House {}/{} - ${:.2}/s",
        house.occupied.len(),
        house.dwellings,
        upkeep.cost_per_second
    );
}

pub fn update_house_text_system(
    query: Query<(&Children, &House, &upkeep::UpkeepCost), Changed<House>>,
    mut house_text_query: Query<&mut Text, With<HouseText>>,
) {
    for (children, house, upkeep) in query.iter() {
        let mut house_text = house_text_query.get_mut(children[0]).unwrap();
        update_house_text(house, upkeep, &mut house_text);
    }
}

pub fn clear_dead_from_house_system(
    mut query: Query<&mut House>,
    potential_occupants: Query<Entity, With<shelter::RequiresHouse>>,
) {
    for mut house in &mut query {
        let mut to_remove = Vec::new();

        for occupant in &house.occupied {
            if potential_occupants.get(*occupant).is_err() {
                to_remove.push(*occupant);
            }
        }

        for occupant in to_remove {
            house.occupied.remove(&occupant);
        }
    }
}

pub fn empty_dilapidated_house_system(
    mut query: Query<&mut House, With<Enum!(buildings::building::BuildingStatus::Dilapidated)>>,
    mut occupants: Query<&mut shelter::RequiresHouse>,
) {
    for mut house in &mut query {
        for occupant in &house.occupied {
            if let Ok(mut occupant) = occupants.get_mut(*occupant) {
                occupant.shelter = None;
            }
        }

        house.occupied.clear();
    }
}
