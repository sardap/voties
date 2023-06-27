use std::{collections::HashSet, time::Duration};

use bevy::{prelude::*, utils::HashMap};
use rand::seq::{IteratorRandom, SliceRandom};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    age::Age,
    buildings::building::{Building, BuildingPlots},
    death::{DeathReason, Mortal},
    energy::Energy,
    hunger::{FoodCollection, FoodPreferences, FoodTemplate, Stomach},
    name,
    reproduction::Reproductive,
    rng,
    shelter::RequiresHouse,
    sim_time::SimTime,
    world_stats::WorldStats,
};

use super::{
    anti_plurality::{self, AntiPluralityResult},
    approval::{Approval, ApprovalResult},
    first_pass_the_post::{FirstPastThePost, FirstPastThePostResult},
    good_ok_bad::{GoodOkBadElection, GoodOkBadResult},
    preferential::Preferential,
    preferential::PreferentialResult,
    star::{Star, StarResult},
    usual_judgment::{self, UsualJudgmentResult},
    voter::Voter,
    voting_methods::OptionRating,
};

#[derive(Debug, Clone)]
pub struct HeldElection {
    pub name: String,
    pub election: Election,
    pub results: Vec<ElectionTypeResult>,
}

impl HeldElection {
    pub fn new(title: &str, election: Election) -> Self {
        let mut results = vec![election.result()];
        for election_type in ElectionType::iter() {
            if election.election_type == election_type {
                continue;
            }
            results.push(election.result_for(election_type));
        }

        Self {
            name: title.to_string(),
            election,
            results,
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct ElectionHistory {
    pub held_elections: Vec<HeldElection>,
}

#[derive(Debug, Resource)]
pub struct ElectionClosedEvent {
    pub held_election: HeldElection,
}

#[derive(Debug, Clone)]
pub enum ElectionTypeResult {
    FirstPastThePostResult(FirstPastThePostResult),
    ApprovalResult(ApprovalResult),
    PreferentialResult(PreferentialResult),
    GoodOkBadResult(GoodOkBadResult),
    StarResult(StarResult),
    AntiPluralityResult(AntiPluralityResult),
    UsualJudgment(UsualJudgmentResult),
}

impl ElectionTypeResult {
    pub fn get_winner(&self) -> &ElectionOption {
        match self {
            ElectionTypeResult::FirstPastThePostResult(result) => &result.winner,
            ElectionTypeResult::ApprovalResult(result) => &result.winner,
            ElectionTypeResult::PreferentialResult(result) => &result.winner,
            ElectionTypeResult::GoodOkBadResult(result) => &result.winner,
            ElectionTypeResult::StarResult(result) => &result.winner,
            ElectionTypeResult::AntiPluralityResult(result) => &result.winner,
            ElectionTypeResult::UsualJudgment(result) => &result.winner,
        }
    }

    pub fn get_type(&self) -> ElectionType {
        match self {
            ElectionTypeResult::FirstPastThePostResult(_) => ElectionType::FirstPastThePost,
            ElectionTypeResult::ApprovalResult(_) => ElectionType::Approval,
            ElectionTypeResult::PreferentialResult(_) => ElectionType::Preferential,
            ElectionTypeResult::GoodOkBadResult(_) => ElectionType::GoodOkBad,
            ElectionTypeResult::StarResult(_) => ElectionType::Star,
            ElectionTypeResult::AntiPluralityResult(_) => ElectionType::AntiPlurality,
            ElectionTypeResult::UsualJudgment(_) => ElectionType::UsualJudgment,
        }
    }
}

pub trait ElectionImpl {
    fn result(
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, EnumIter)]
pub enum ElectionType {
    FirstPastThePost,
    Approval,
    Preferential,
    GoodOkBad,
    Star,
    AntiPlurality,
    UsualJudgment,
}

impl ElectionType {
    pub fn result(
        &self,
        options: &[ElectionOption],
        option_ratings: &Vec<&Vec<OptionRating>>,
    ) -> ElectionTypeResult {
        match self {
            ElectionType::FirstPastThePost => FirstPastThePost::result(options, option_ratings),
            ElectionType::Approval => Approval::result(options, option_ratings),
            ElectionType::Preferential => Preferential::result(options, option_ratings),
            ElectionType::GoodOkBad => GoodOkBadElection::result(options, option_ratings),
            ElectionType::Star => Star::result(options, option_ratings),
            ElectionType::AntiPlurality => anti_plurality::result(options, option_ratings),
            ElectionType::UsualJudgment => usual_judgment::result(options, option_ratings),
        }
    }
}

impl ToString for ElectionType {
    fn to_string(&self) -> String {
        match self {
            ElectionType::FirstPastThePost => "First Past The Post".to_string(),
            ElectionType::Approval => "Approval".to_string(),
            ElectionType::Preferential => "Preferential".to_string(),
            ElectionType::GoodOkBad => "Good Ok Bad".to_string(),
            ElectionType::Star => "Star".to_string(),
            ElectionType::AntiPlurality => "Anti Plurality".to_string(),
            ElectionType::UsualJudgment => "Usual Judgment".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ElectionOption {
    DoNothing,
    MakeFarm(FoodTemplate),
    MakeRz,
    MoneyHole,
    Mint,
    House(i32),
}

impl ToString for ElectionOption {
    fn to_string(&self) -> String {
        match self {
            ElectionOption::DoNothing => "Do Nothing".to_owned(),
            ElectionOption::MakeFarm(food_template) => {
                format!("Make \"{}\" Farm", food_template.name)
            }
            ElectionOption::MakeRz => "Make a bone zone".to_string(),
            ElectionOption::MoneyHole => format!("Make a money hole"),
            ElectionOption::Mint => format!("Make a mint"),
            ElectionOption::House(dwellings) => format!("Make a {} bedroom house", dwellings),
        }
    }
}

#[derive(Debug, Clone, Component)]
pub struct Election {
    pub options: Vec<ElectionOption>,
    pub election_type: ElectionType,
    pub votes: HashMap<Entity, Vec<OptionRating>>,
    pub time_open: Duration,
}

impl Election {
    fn get_option_ratings(&self) -> Vec<&Vec<OptionRating>> {
        let option_ratings = self.votes.values().collect::<Vec<_>>();
        option_ratings
    }

    pub fn result_for(&self, election_type: ElectionType) -> ElectionTypeResult {
        let option_ratings = self.get_option_ratings();
        election_type.result(&self.options, &option_ratings)
    }

    pub fn result(&self) -> ElectionTypeResult {
        self.result_for(self.election_type)
    }
}

#[derive(Debug)]
pub struct VoterAttributes<'a> {
    pub voter: &'a Voter,
    pub energy: Option<&'a Energy>,
    pub stomach: Option<&'a Stomach>,
    pub food_preferences: Option<&'a FoodPreferences>,
    pub reproductive: Option<&'a Reproductive>,
    pub housing: Option<&'a RequiresHouse>,
}

pub mod want_level {
    pub const EXTREMELY_NEGATIVE: i32 = -30;
    pub const NEGATIVE: i32 = -20;
    pub const SLIGHTLY_NEGATIVE: i32 = -10;
    pub const NEUTRAL: i32 = 0;
    pub const SLIGHTLY_POSITIVE: i32 = 10;
    pub const POSITIVE: i32 = 20;
    pub const EXTREMELY_POSITIVE: i32 = 30;
}

fn modify_rating(
    rng: &mut impl rand::Rng,
    rating: i32,
    option: &ElectionOption,
    attributes: &VoterAttributes,
) -> i32 {
    let modifier = match option {
        ElectionOption::DoNothing => return 0,
        ElectionOption::MakeFarm(_) => attributes.voter.food_care,
        ElectionOption::MakeRz => attributes.voter.reproductive_care,
        ElectionOption::MoneyHole => attributes.voter.money_care,
        ElectionOption::Mint => attributes.voter.money_care,
        ElectionOption::House(_) => attributes.voter.housing_care,
    };

    let mut rating = rating;
    // Sprinkle some randomness
    rating += rng.gen_range(want_level::SLIGHTLY_NEGATIVE..want_level::SLIGHTLY_POSITIVE);
    rating += modifier;

    rating
}

impl Election {
    pub fn vote(
        &mut self,
        rng: &mut impl rand::Rng,
        voter: Entity,
        attributes: VoterAttributes,
        stats: &WorldStats,
    ) {
        if self.votes.contains_key(&voter) {
            return;
        }

        let mut option_ratings = vec![];

        for (index, option) in self.options.iter().enumerate() {
            let rating = match option {
                ElectionOption::DoNothing => want_level::NEUTRAL,
                ElectionOption::MakeFarm(food_template) => {
                    let mut rating = want_level::NEUTRAL;

                    if let Some(energy) = attributes.energy {
                        if energy.current_kcal < energy.max_kcal * 0.3 {
                            rating = want_level::POSITIVE;
                        }
                    }

                    if let Some(food_preferences) = attributes.food_preferences {
                        for food_group in &food_template.groups {
                            if food_preferences.prefers.contains(&food_group) {
                                rating = want_level::SLIGHTLY_POSITIVE;
                            }
                        }

                        if !food_preferences.will_eat(&food_template.get_food()) {
                            rating = want_level::EXTREMELY_NEGATIVE;
                        }
                    }

                    rating
                }
                ElectionOption::MakeRz => {
                    let mut rating = want_level::NEUTRAL;

                    if let Some(reproductive) = attributes.reproductive {
                        if reproductive.wants_to_reproduce() {
                            rating += want_level::SLIGHTLY_POSITIVE;
                        }
                    }

                    rating
                }
                ElectionOption::MoneyHole => {
                    let filled_percentage: f64 = stats.hole_filled_capacity.max();

                    if filled_percentage > 0.9 {
                        want_level::SLIGHTLY_POSITIVE
                    } else {
                        want_level::NEUTRAL
                    }
                }
                ElectionOption::Mint => {
                    let filled_percentage: f64 = stats.hole_filled_capacity.average();

                    if filled_percentage < 0.3 {
                        want_level::POSITIVE
                    } else if filled_percentage < 0.5 {
                        want_level::SLIGHTLY_POSITIVE
                    } else {
                        want_level::NEUTRAL
                    }
                }
                ElectionOption::House(_dwellings) => {
                    let mut rating = want_level::NEUTRAL;

                    if let Some(housing) = attributes.housing {
                        if housing.shelter.is_none() {
                            rating = want_level::POSITIVE;
                        }
                    }

                    if stats.houses_filled.max() > 0.9 {
                        rating += want_level::SLIGHTLY_POSITIVE;
                    }

                    rating
                }
            };

            option_ratings.push(OptionRating {
                option_index: index,
                rating,
            });
        }

        let total_pop: f64 = (stats.population.latest() + stats.deaths.sum()) as f64;
        // Plus to options based on the deaths by inaction
        for option in &mut option_ratings {
            let death_count = match self.options[option.option_index] {
                ElectionOption::DoNothing => 0,
                ElectionOption::MakeFarm(_) => stats.deaths.get(&DeathReason::Starvation),
                // Voties can't die form horniness but maybe they should
                ElectionOption::MakeRz => 0,
                // People don't die directly from running out of money
                ElectionOption::MoneyHole | ElectionOption::Mint => 0,
                //
                ElectionOption::House(_) => stats.deaths.get(&DeathReason::Homeliness),
            };

            if death_count == 0 {
                continue;
            }
            let percent_died_of = death_count as f64 / total_pop;
            let care = attributes.voter.death_care as f64 * percent_died_of;
            option.rating += care.ceil() as i32;
        }

        // Apply modifiers
        for option in &mut option_ratings {
            option.rating = modify_rating(
                rng,
                option.rating,
                &self.options[option.option_index],
                &attributes,
            );
        }

        option_ratings.sort_by(|a, b| a.rating.cmp(&b.rating).reverse());

        self.votes.insert(voter, option_ratings);
    }
}

#[derive(Bundle)]
pub struct ElectionBundle {
    pub name: name::Name,
    pub election: Election,
}

#[derive(Debug, Resource)]
pub struct ElectionTimer(pub Timer);

pub fn create_election(
    commands: &mut Commands,
    election_type: ElectionType,
    rng: &mut impl rand::Rng,
    food_collection: &FoodCollection,
) {
    let options = get_options(rng, &food_collection);

    info!("About to create an election {:?}", options);

    commands.spawn(ElectionBundle {
        name: name::Name(format!("Election")),
        election: Election {
            options,
            election_type,
            votes: default(),
            time_open: default(),
        },
    });
}

pub fn start_election_system(
    mut commands: Commands,
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut rng: ResMut<rng::Rng>,
    mut timer: ResMut<ElectionTimer>,
    food_collection: Res<FoodCollection>,
) {
    if !timer.0.tick(sim_time.delta(&time)).just_finished() {
        return;
    }

    let election = ElectionType::iter().choose(&mut rng.inner).unwrap();

    create_election(&mut commands, election, &mut rng.inner, &food_collection);
}

pub fn close_elections_system(
    mut commands: Commands,
    time: Res<Time>,
    sim_time: Res<SimTime>,
    mut election_history: ResMut<ElectionHistory>,
    asset_server: Res<AssetServer>,
    mut plots: ResMut<BuildingPlots>,
    mut rng: ResMut<rng::Rng>,
    mut closed_election_events: EventWriter<ElectionClosedEvent>,
    mut query: Query<(Entity, &mut Election, &name::Name)>,
) {
    for (entity, mut election, name) in &mut query {
        election.time_open += sim_time.delta(&time);

        if election.time_open <= Duration::from_secs(15) {
            continue;
        }

        if election.votes.len() <= 0 {
            commands.entity(entity).despawn_recursive();
            return;
        }

        let result = election.result();
        info!("Election result: {:?}", result);
        for election_type in ElectionType::iter() {
            info!(
                "Election type: {:?} Winner: {:?}",
                election_type,
                election.result_for(election_type).get_winner()
            );
        }

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
            ElectionOption::House(dwellings) => Building::House(*dwellings).build(
                &mut commands,
                &asset_server,
                &mut plots,
                &mut rng.inner,
            ),
            ElectionOption::DoNothing => {
                info!("Apathy won!")
            }
        };

        let held_election = HeldElection::new(name.0.as_str(), election.clone());
        closed_election_events.send(ElectionClosedEvent {
            held_election: held_election.clone(),
        });
        election_history.held_elections.push(held_election);

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

fn get_options(rng: &mut impl rand::Rng, food_collection: &FoodCollection) -> Vec<ElectionOption> {
    let mut result: Vec<ElectionOption> = vec![];

    let dwellings_count = rng.gen_range(3..=10);

    result.push(ElectionOption::DoNothing);
    result.push(ElectionOption::House(dwellings_count));
    result.push(ElectionOption::MakeFarm(
        food_collection.foods.choose(rng).unwrap().clone(),
    ));
    result.push(ElectionOption::MoneyHole);
    result.push(ElectionOption::Mint);

    result
}
