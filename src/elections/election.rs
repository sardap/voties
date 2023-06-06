use std::{collections::HashSet, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use rand::{seq::SliceRandom, Rng};

use crate::{
    age::Age,
    building::{Building, BuildingPlots},
    death::{DeathReason, Mortal},
    energy::Energy,
    hunger::{FoodCollection, FoodPreferences, FoodTemplate},
    name,
    reproduction::Reproductive,
    rng,
    ui::resources::BuildingButtonNode,
};

use super::{
    first_pass_the_post::{FirstPastThePost, FirstPastThePostResult},
    voter::Voter,
    voting_methods::OptionRating,
};

#[derive(Debug, Resource, Default)]
pub struct ElectionHistory {
    pub results: Vec<ElectionResult>,
}

#[derive(Debug)]
pub struct ElectionResult {
    pub name: String,
    pub options: Vec<ElectionOption>,
    pub result: ElectionTypeResult,
}

#[derive(Debug)]
pub enum ElectionTypeResult {
    FirstPastThePostResult(FirstPastThePostResult),
}

impl ElectionTypeResult {
    pub fn get_winner(&self) -> &ElectionOption {
        match self {
            ElectionTypeResult::FirstPastThePostResult(result) => &result.winner,
        }
    }
}

pub trait ElectionImpl {
    fn vote(&mut self, option_ratings: &[OptionRating]);
    fn result(&self, options: &[ElectionOption]) -> ElectionTypeResult;
}

#[derive(Debug, Clone)]
pub enum ElectionType {
    FirstPastThePost(FirstPastThePost),
}

impl Default for ElectionType {
    fn default() -> Self {
        ElectionType::FirstPastThePost(FirstPastThePost::default())
    }
}

impl ToString for ElectionType {
    fn to_string(&self) -> String {
        match self {
            ElectionType::FirstPastThePost(_) => "First Past The Post".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElectionOption {
    MakeFarm(FoodTemplate),
    MakeRz,
}

impl ToString for ElectionOption {
    fn to_string(&self) -> String {
        match self {
            ElectionOption::MakeFarm(food_template) => {
                format!("Make \"{}\" Farm", food_template.name)
            }
            ElectionOption::MakeRz => "Make a bone zone".to_string(),
        }
    }
}

#[derive(Debug, Component, Default)]
pub struct Election {
    pub options: Vec<ElectionOption>,
    pub election_type: ElectionType,
    pub voted: HashSet<Entity>,
    pub time_open: Duration,
}

#[derive(Debug)]
pub struct VoterAttributes<'a> {
    pub energy: Option<&'a Energy>,
    pub food_preferences: Option<&'a FoodPreferences>,
    pub reproductive: Option<&'a Reproductive>,
}

impl Election {
    pub fn vote(&mut self, voter: Entity, attributes: VoterAttributes) {
        if self.voted.contains(&voter) {
            return;
        }

        self.voted.insert(voter);

        let mut option_ratings = vec![];

        for (index, option) in self.options.iter().enumerate() {
            let rating = match option {
                ElectionOption::MakeFarm(food_template) => {
                    let mut rating = 0;

                    if let Some(energy) = attributes.energy {
                        if energy.current_kcal < energy.max_kcal * 0.5 {
                            rating += 1;
                        }
                    }

                    if let Some(food_preferences) = attributes.food_preferences {
                        if !food_preferences.will_eat(&food_template.get_food()) {
                            rating = 0;
                        }
                    }

                    rating
                }
                ElectionOption::MakeRz => {
                    let mut rating = 0;

                    if let Some(reproductive) = attributes.reproductive {
                        if reproductive.wants_to_reproduce() {
                            rating += 1;
                        }
                    }

                    rating
                }
            };

            option_ratings.push(OptionRating {
                option_index: index,
                rating,
            });
        }

        option_ratings.sort_by(|a, b| a.rating.cmp(&b.rating));

        match &mut self.election_type {
            ElectionType::FirstPastThePost(election) => election.vote(&option_ratings),
        }
    }
}

#[derive(Bundle, Default)]
pub struct ElectionBundle {
    pub name: name::Name,
    pub election: Election,
}

#[derive(Debug, Resource)]
pub struct ElectionTimer(pub Timer);

pub fn create_election(
    commands: &mut Commands,
    time: &Time,
    election_type: ElectionType,
    rng: &mut impl rand::Rng,
    food_collection: &FoodCollection,
    query: &Query<
        (
            Option<&Reproductive>,
            Option<&Age>,
            Option<&Mortal>,
            Option<&Energy>,
        ),
        With<Voter>,
    >,
) {
    let options = get_options(time, rng, &food_collection, &query);

    info!("About to create an election {:?}", options);

    commands.spawn(ElectionBundle {
        name: name::Name(format!("Cycles {}", time.elapsed_seconds().floor())),
        election: Election {
            options,
            election_type,
            ..default()
        },
    });
}

pub fn start_election_system(
    mut commands: Commands,
    time: Res<Time>,
    mut rng: ResMut<rng::Rng>,
    mut timer: ResMut<ElectionTimer>,
    food_collection: Res<FoodCollection>,
    query: Query<
        (
            Option<&Reproductive>,
            Option<&Age>,
            Option<&Mortal>,
            Option<&Energy>,
        ),
        With<Voter>,
    >,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }

    let election = match rng.inner.gen_range(0..1) {
        0 => ElectionType::FirstPastThePost(FirstPastThePost::default()),
        _ => panic!("invalid election type"),
    };

    create_election(
        &mut commands,
        &time,
        election,
        &mut rng.inner,
        &food_collection,
        &query,
    );
}

pub fn close_elections_system(
    mut commands: Commands,
    time: Res<Time>,
    mut election_history: ResMut<ElectionHistory>,
    asset_server: Res<AssetServer>,
    mut plots: ResMut<BuildingPlots>,
    mut rng: ResMut<rng::Rng>,
    mut query: Query<(Entity, &mut Election, &name::Name)>,
) {
    for (entity, mut election, name) in &mut query {
        election.time_open += time.delta();

        if election.time_open <= Duration::from_secs(15) {
            continue;
        }

        let options = election.options.clone();

        let result = match &election.election_type {
            ElectionType::FirstPastThePost(x) => x.result(&options),
        };

        match result.get_winner() {
            ElectionOption::MakeFarm(food_template) => Building::Farm(food_template.clone()).build(
                &mut commands,
                &asset_server,
                &mut plots,
                &mut rng.inner,
            ),
            ElectionOption::MakeRz => Building::ReproductiveZone.build(
                &mut commands,
                &asset_server,
                &mut plots,
                &mut rng.inner,
            ),
        };

        election_history.results.push(ElectionResult {
            name: name.0.clone(),
            result,
            options,
        });

        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Want {
    Food,
    Reproduction,
}

impl Default for Want {
    fn default() -> Self {
        Want::Food
    }
}

fn add_to_tally(tally: &mut HashMap<Want, usize>, want: Want) {
    if let Some(count) = tally.get_mut(&want) {
        *count += 1;
    } else {
        tally.insert(want, 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WantTally {
    want: Want,
    count: usize,
}

fn get_options(
    time: &Time,
    rng: &mut impl rand::Rng,
    food_collection: &FoodCollection,
    query: &Query<
        (
            Option<&Reproductive>,
            Option<&Age>,
            Option<&Mortal>,
            Option<&Energy>,
        ),
        With<Voter>,
    >,
) -> Vec<ElectionOption> {
    let mut wants = HashMap::new();

    for (_reproductive, _age, mortal, energy) in query {
        if let Some(mortal) = mortal {
            for risk in &mortal.at_risk {
                match risk {
                    DeathReason::Starvation => {
                        add_to_tally(&mut wants, Want::Food);
                    }
                    _ => {}
                }
            }
        }

        if let Some(energy) = energy {
            if energy.current_kcal < energy.max_kcal * 0.5 {
                add_to_tally(&mut wants, Want::Food);
            }
        }

        // if let Some(reproductive) = reproductive {
        //     if reproductive.next_reproduction > time.elapsed() {
        //         add_to_tally(&mut wants, Want::Reproduction);
        //     }
        // }
    }

    let mut wants: Vec<WantTally> = wants
        .into_iter()
        .map(|(want, count)| WantTally { want, count })
        .collect();
    wants.sort_by(|a, b| a.count.cmp(&b.count));

    let want_count = wants.iter().map(|i| i.count).sum::<usize>();

    let voter_count = query.iter().count();

    let option_count = want_count.min(6).min(voter_count);

    let mut options = HashSet::new();

    for _ in 0..option_count {
        let want_index = rng.gen_range(0..want_count);
        let selected_want = match wants
            .iter()
            .enumerate()
            .find(|(_, want)| want_index < want.count)
        {
            Some(foo) => foo.1.want,
            None => {
                continue;
            }
        };

        let option: ElectionOption = match selected_want {
            Want::Food => {
                ElectionOption::MakeFarm(food_collection.foods.choose(rng).unwrap().clone())
            }
            Want::Reproduction => ElectionOption::MakeRz,
        };

        options.insert(option);
    }

    if options.is_empty() {
        options.insert(ElectionOption::MakeFarm(
            food_collection.foods.choose(rng).unwrap().clone(),
        ));
    }

    options.into_iter().collect()
}
