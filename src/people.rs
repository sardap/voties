use std::{collections::HashSet, time::Duration};

use crate::{
    age, assets, brain, collision, death,
    elections::voter::Voter,
    energy,
    grave::GraveBundle,
    hunger::{self, Stomach},
    movement::{self},
    name,
    reproduction::{self},
    rng,
};
use bevy::prelude::*;
use rand::{seq::IteratorRandom, Rng};
use strum::IntoEnumIterator;

fn create_person_stomach(
    rng: &mut impl rand::Rng,
    range: (measurements::Volume, measurements::Volume),
) -> hunger::Stomach {
    let stomach_size = measurements::Volume::from_milliliters(
        rng.gen_range(range.0.as_milliliters()..range.1.as_milliliters()),
    );

    let filled =
        measurements::Volume::from_milliliters(rng.gen_range(0.0..stomach_size.as_milliliters()));

    hunger::Stomach::new(stomach_size, filled)
}

pub fn wont_eat_count(rng: &mut impl rand::Rng) -> usize {
    if rng.gen::<u32>() % 3 == 0 {
        let weight = rng.gen_range(0..100);
        if weight < 50 {
            1
        } else if weight < 75 {
            2
        } else {
            3
        }
    } else {
        0
    }
}

pub fn fill_wont_eat(
    count: usize,
    wont_eat: &mut HashSet<hunger::FoodGroup>,
    rng: &mut impl rand::Rng,
) {
    let food_groups: Vec<_> = hunger::FoodGroup::iter().collect();

    while wont_eat.len() < count {
        let food_group = food_groups[rng.gen_range(0..food_groups.len())];
        wont_eat.insert(food_group);
    }
}

pub fn create_person(
    commands: &mut Commands,
    time: &Time,
    asset_server: &AssetServer,
    rng: &mut impl rand::Rng,
    name: &str,
    spawn_location: Vec3,
    stomach_size_range: (measurements::Volume, measurements::Volume),
    speed: f32,
    age: Duration,
    wont_eat_groups: &[hunger::FoodGroup],
) {
    let person_entity = commands
        .spawn((
            SpriteBundle {
                texture: asset_server.load(assets::DEFAULT_PERSON_SPRITE_PATH),
                transform: Transform::from_translation(spawn_location),
                ..default()
            },
            create_person_stomach(rng, stomach_size_range),
            energy::Energy {
                current_kcal: rng.gen_range(1000.0..2500.0),
                max_kcal: 2500.0,
            },
            hunger::FoodPreferences::new(wont_eat_groups),
            movement::MovementGoal::default(),
            movement::MovementSpeed(speed),
            movement::Velocity::default(),
            Person::default(),
            collision::Collider,
            collision::CollisionHolder::default(),
            death::Mortal::default(),
            age::Age {
                duration_alive: age,
            },
            brain::Brain,
            name::Name(name.to_string()),
            reproduction::Reproductive {
                next_reproduction: Duration::from_secs(rng.gen_range(
                    time.elapsed().as_secs()..(time.elapsed() + Duration::from_secs(30)).as_secs(),
                )),
            },
        ))
        .id();

    commands.entity(person_entity).insert(Voter);

    let info_text = commands
        .spawn(Text2dBundle {
            text: Text::from_section(
                "Example Text",
                TextStyle {
                    font: asset_server.load(assets::DEFAULT_FONT_PATH),
                    font_size: 10.0,
                    color: Color::BLACK,
                },
            ),
            transform: Transform::from_xyz(0.0, 40.0, 10.0),
            ..default()
        })
        .id();

    commands.entity(person_entity).push_children(&[info_text]);
}

#[derive(Debug, Component, Default)]
pub struct Person;

pub fn update_info_text(
    q_parent: Query<
        (
            &death::Mortal,
            &name::Name,
            &Children,
            &energy::Energy,
            &Stomach,
            &age::Age,
            Option<&reproduction::Pregnant>,
        ),
        With<Person>,
    >,
    mut q_child: Query<&mut Text>,
) {
    for (mortal, name, children, energy, stomach, age, pregnant) in &q_parent {
        let mut info_text = q_child.get_mut(children[0]).unwrap();

        let mut info = string_builder::Builder::default();
        info.append("Identifier: ");
        info.append(name.0.as_str());
        info.append("\n");

        info.append("Age: ");
        info.append(age.duration_alive.as_secs().to_string());
        info.append(" Cycles\n");

        info.append("Stomach: ");
        info.append(stomach.max_size_ml.round().to_string());
        info.append("ml ");
        info.append(
            (stomach.filled_ml / stomach.max_size_ml * 100.0)
                .round()
                .to_string(),
        );
        info.append("%\n");

        info.append("Energy: ");
        info.append((energy.current_kcal).round().to_string());
        info.append("kcal\n");

        if let Some(_) = pregnant {
            info.append("Pregnant!\n");
        }

        if mortal.at_risk.len() > 0 {
            info.append("At risk:\n");
            for reason in &mortal.at_risk {
                info.append("* ");
                info.append(reason.get_at_risk_message().as_str());
                info.append("\n");
            }
        }

        info_text.sections[0].value = info.string().unwrap();
    }
}

pub fn create_grave_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    query: Query<(&death::Mortal, &Transform, &name::Name, &age::Age), With<Person>>,
) {
    for (mortal, source_position, name, age) in &query {
        if let Some(reason) = &mortal.dead {
            commands.spawn(GraveBundle::new(
                &asset_server,
                &source_position,
                &time,
                &name.0,
                reason.get_reason_of_death_message().as_str(),
                age.duration_alive,
            ));
        }
    }
}

pub fn give_birth_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut rng: ResMut<rng::Rng>,
    mut name_gen: ResMut<name::NameGenerator>,
    parents_q: Query<
        (
            &hunger::Stomach,
            &movement::MovementSpeed,
            &hunger::FoodPreferences,
        ),
        With<Person>,
    >,
    mut pregnant_q: Query<
        (
            Entity,
            &reproduction::Pregnant,
            &Transform,
            &mut energy::Energy,
        ),
        With<Person>,
    >,
) {
    for (entity, pregnant, trans, mut energy) in &mut pregnant_q {
        if time.elapsed() - pregnant.pregnant_since > pregnant.pregnancy_duration {
            let min_stomach_range;
            let max_stomach_range;
            {
                let parents_stomach: Vec<_> = pregnant
                    .parents
                    .iter()
                    .map(|i| parents_q.get_component::<hunger::Stomach>(*i))
                    .filter_map(|i| i.is_ok().then(|| i.unwrap()))
                    .collect();

                min_stomach_range = parents_stomach
                    .iter()
                    .map(|s| s.max_size_ml)
                    .min_by(|a, b| a.total_cmp(b))
                    .unwrap()
                    * 0.9;

                max_stomach_range = parents_stomach
                    .iter()
                    .map(|s| s.max_size_ml)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap()
                    * 1.1;
            }

            let speed;
            {
                let parents_speed: Vec<_> = pregnant
                    .parents
                    .iter()
                    .map(|i| parents_q.get_component::<movement::MovementSpeed>(*i))
                    .filter_map(|i| i.is_ok().then(|| i.unwrap()))
                    .collect();

                let min_speed = parents_speed
                    .iter()
                    .map(|s| s.0)
                    .min_by(|a, b| a.total_cmp(b))
                    .unwrap()
                    * 0.9;

                let max_speed = parents_speed
                    .iter()
                    .map(|s| s.0)
                    .max_by(|a, b| a.total_cmp(b))
                    .unwrap()
                    * 1.1;

                speed = rng.inner.gen_range(min_speed..max_speed);
            }

            let wont_eat_groups: Vec<_>;
            {
                let parents_food_groups: Vec<_> = pregnant
                    .parents
                    .iter()
                    .map(|i| parents_q.get_component::<hunger::FoodPreferences>(*i))
                    .filter_map(|i| i.is_ok().then(|| i.unwrap()))
                    .collect();

                let complete_set = parents_food_groups
                    .iter()
                    .map(|p| p.wont_eat.iter())
                    .flatten()
                    .collect::<HashSet<_>>();

                let food_groups: Vec<_> = hunger::FoodGroup::iter().collect();
                let food_groups_count = food_groups.len();

                let wont_eat_count = if complete_set.len() == 0 {
                    rng.inner
                        .gen_range(0..(complete_set.len() + 1).min(food_groups_count))
                } else {
                    wont_eat_count(&mut rng.inner)
                };

                let mut wont_eat = HashSet::new();
                while wont_eat.len() < wont_eat_count {
                    let food_group = hunger::FoodGroup::iter()
                        .filter(|fg| !wont_eat.contains(fg))
                        .choose(&mut rng.inner)
                        .unwrap();
                    wont_eat.insert(food_group);
                }

                fill_wont_eat(wont_eat_count, &mut wont_eat, &mut rng.inner);

                wont_eat_groups = wont_eat.into_iter().collect();
            }

            energy.use_kcal(1000.0);

            create_person(
                &mut commands,
                &time,
                &asset_server,
                &mut rng.inner,
                name_gen.generate().as_str(),
                trans.translation,
                (
                    measurements::Volume::from_milliliters(min_stomach_range),
                    measurements::Volume::from_milliliters(max_stomach_range),
                ),
                speed,
                Duration::ZERO,
                &wont_eat_groups,
            );

            commands.entity(entity).remove::<reproduction::Pregnant>();
        }
    }
}
