use std::{collections::HashSet, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use rand::{seq::SliceRandom, Rng};

use crate::{
    age::Age,
    building::{Building, BuildingPlots},
    death::{DeathReason, Mortal},
    energy::Energy,
    hunger::{FoodCollection, FoodPreferences, FoodTemplate},
    money::Treasury,
    name,
    reproduction::Reproductive,
    rng,
    world_stats::WorldStats,
};

use super::{
    approval::{Approval, ApprovalResult},
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
    ApprovalResult(ApprovalResult),
}

impl ElectionTypeResult {
    pub fn get_winner(&self) -> &ElectionOption {
        match self {
            ElectionTypeResult::FirstPastThePostResult(result) => &result.winner,
            ElectionTypeResult::ApprovalResult(result) => &result.winner,
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
    Approval(Approval),
}

impl ElectionType {
    pub fn vote(&mut self, option_ratings: &[OptionRating]) {
        match self {
            ElectionType::FirstPastThePost(election) => election.vote(option_ratings),
            ElectionType::Approval(election) => election.vote(option_ratings),
        }
    }

    pub fn result(&self, options: &[ElectionOption]) -> ElectionTypeResult {
        match self {
            ElectionType::FirstPastThePost(election) => election.result(options),
            ElectionType::Approval(election) => election.result(options),
        }
    }
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
            ElectionType::Approval(_) => "Approval".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElectionOption {
    MakeFarm(FoodTemplate),
    MakeRz,
    MoneyHole,
    Mint,
}

impl ToString for ElectionOption {
    fn to_string(&self) -> String {
        match self {
            ElectionOption::MakeFarm(food_template) => {
                format!("Make \"{}\" Farm", food_template.name)
            }
            ElectionOption::MakeRz => "Make a bone zone".to_string(),
            ElectionOption::MoneyHole => format!("Make a money hole"),
            ElectionOption::Mint => format!("Make a mint"),
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
    pub voter: &'a Voter,
    pub energy: Option<&'a Energy>,
    pub food_preferences: Option<&'a FoodPreferences>,
    pub reproductive: Option<&'a Reproductive>,
}

mod want_level {
    pub const NONE: i32 = 0;
    pub const LOW: i32 = 10;
    pub const SLIGHTLY: i32 = 20;
    pub const MEDIUM: i32 = 50;
    pub const HIGH: i32 = 100;
    pub const VERY_HIGH: i32 = 200;
}

impl Election {
    pub fn vote(&mut self, voter: Entity, attributes: VoterAttributes, stats: &WorldStats) {
        if self.voted.contains(&voter) {
            return;
        }

        self.voted.insert(voter);

        let mut option_ratings = vec![];

        for (index, option) in self.options.iter().enumerate() {
            let rating = match option {
                ElectionOption::MakeFarm(food_template) => {
                    let mut rating = want_level::NONE;

                    if let Some(energy) = attributes.energy {
                        if energy.current_kcal < energy.max_kcal * 0.3 {
                            rating += want_level::MEDIUM;
                        }
                    }

                    if let Some(food_preferences) = attributes.food_preferences {
                        for food_group in &food_template.groups {
                            if food_preferences.prefers.contains(&food_group) {
                                rating += want_level::LOW;
                            }
                        }

                        if !food_preferences.will_eat(&food_template.get_food()) {
                            rating = want_level::NONE;
                        }
                    }

                    rating
                }
                ElectionOption::MakeRz => {
                    let mut rating = want_level::NONE;

                    if let Some(reproductive) = attributes.reproductive {
                        if reproductive.wants_to_reproduce() {
                            rating += want_level::HIGH;
                        }
                    }

                    rating
                }
                ElectionOption::MoneyHole => {
                    let filled_percentage: f64 = stats.hole_filled_capacity.average();

                    if filled_percentage > 0.9 {
                        want_level::HIGH
                    } else if filled_percentage > 0.7 {
                        want_level::SLIGHTLY
                    } else {
                        want_level::NONE
                    }
                }
                ElectionOption::Mint => {
                    let building_count = stats.buildings.count();
                    let dilapidated_count = stats
                        .buildings
                        .get(&crate::building::BuildingStatus::Dilapidated);

                    let percentage_dilapidated = dilapidated_count as f64 / building_count as f64;

                    if percentage_dilapidated > 0.8 {
                        want_level::VERY_HIGH
                    } else {
                        want_level::NONE
                    }
                }
            };

            option_ratings.push(OptionRating {
                option_index: index,
                rating,
            });
        }

        // Apply modifiers
        for option in &mut option_ratings {
            let modifier = match self.options[option.option_index] {
                ElectionOption::MakeFarm(_) => attributes.voter.food_care,
                ElectionOption::MakeRz => attributes.voter.reproductive_care,
                ElectionOption::MoneyHole => attributes.voter.money_care,
                ElectionOption::Mint => attributes.voter.money_care,
            };

            option.rating = (option.rating as f32 * modifier) as i32;
        }

        option_ratings.sort_by(|a, b| a.rating.cmp(&b.rating).reverse());

        self.election_type.vote(&option_ratings);
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

    let election = match rng.inner.gen_range(0..2) {
        0 => ElectionType::FirstPastThePost(FirstPastThePost::default()),
        1 => ElectionType::Approval(Approval::default()),
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

        let result = election.election_type.result(&options);

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
            ElectionOption::MoneyHole => {
                Building::MoneyHole.build(&mut commands, &asset_server, &mut plots, &mut rng.inner)
            }
            ElectionOption::Mint => {
                Building::Mint.build(&mut commands, &asset_server, &mut plots, &mut rng.inner)
            }
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

    let option_count = want_count.min(3).min(voter_count);

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

    let mut result: Vec<ElectionOption> = options.into_iter().collect();

    result.push(ElectionOption::MoneyHole);
    result.push(ElectionOption::Mint);

    result
}
