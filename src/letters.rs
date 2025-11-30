use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_shuffle_bag::ShuffleBag;
use rand::{Rng, seq::IteratorRandom};
use serde::Deserialize;

use crate::{
    constants::TIME_LIMIT_RANGE, rng::RngResource, scene::spawning::WordCube, states::GameStates,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(RonAssetPlugin::<TestimonialStub>::new(&[
        "blessing.ron",
        "curse.ron",
        "decoy.ron",
    ]))
    .add_plugins(RonAssetPlugin::<Flavor>::new(&["flavor.ron"]))
    .add_plugins(RonAssetPlugin::<Name>::new(&["name.ron"]))
    .add_systems(OnExit(GameStates::Loading), create_letter_bag_system);
}

#[derive(AssetCollection, Resource)]
pub struct LetterAssets {
    #[asset(key = "letters.blessings", collection(typed))]
    pub blessings: Vec<Handle<TestimonialStub>>,
    #[asset(key = "letters.curses", collection(typed))]
    pub curses: Vec<Handle<TestimonialStub>>,
    #[asset(key = "letters.decoys", collection(typed))]
    pub decoys: Vec<Handle<TestimonialStub>>,
    #[asset(key = "letters.flavors", collection(typed))]
    pub flavors: Vec<Handle<Flavor>>,
    #[asset(key = "letters.names", collection(typed))]
    pub names: Vec<Handle<Name>>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Effect {
    Score(i32),
    Money(i32),
    Income(i32),
    Noop,
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Score(score) => write!(f, "{score} score"),
            Effect::Money(money) => write!(f, "{money} money"),
            Effect::Income(income) => write!(f, "{income} income"),
            Effect::Noop => write!(f, ""),
        }
    }
}

#[derive(Debug, Deserialize, Asset, TypePath, Clone)]
pub struct TestimonialStub {
    pub message: String,
    pub effect: Effect,
    pub targets: Vec<usize>,
}

#[derive(Debug, Deserialize, Asset, TypePath, Clone)]
pub struct Flavor {
    pub title: String,
    pub body: String,
    pub signoff: String,
    pub footer: String,
}

#[derive(Debug, Deserialize, Asset, TypePath, Clone)]
pub struct Name {
    pub first_name: String,
    pub pronouns: Pronouns,
}

#[derive(Debug, Deserialize, Clone)]
pub enum Pronouns {
    HeHimHis,
    SheHerHers,
    TheyThemTheir,
}

impl Pronouns {
    pub fn subject(&self) -> String {
        match self {
            Pronouns::HeHimHis => "he".to_string(),
            Pronouns::SheHerHers => "she".to_string(),
            Pronouns::TheyThemTheir => "they".to_string(),
        }
    }

    pub fn object(&self) -> String {
        match self {
            Pronouns::HeHimHis => "him".to_string(),
            Pronouns::SheHerHers => "her".to_string(),
            Pronouns::TheyThemTheir => "them".to_string(),
        }
    }

    pub fn possessive(&self) -> String {
        match self {
            Pronouns::HeHimHis => "his".to_string(),
            Pronouns::SheHerHers => "her".to_string(),
            Pronouns::TheyThemTheir => "their".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Testimonial {
    pub first_name: String,
    pub last_initial: char,
    pub pronouns: Pronouns,
    pub message: String,
    pub target_word: String,
    pub effect: Effect,
    pub collected: bool,
}

#[derive(Debug, Clone)]
pub struct InterpolatedFlavor {
    pub body: String,
    pub signoff: String,
}

#[derive(Debug, Clone)]
pub struct Letter {
    pub flavor: Flavor,
    pub interpolated_flavor: InterpolatedFlavor,
    pub recipients: usize,
    pub time_limit: usize,
    pub blessings: Vec<Testimonial>,
    pub curses: Vec<Testimonial>,
}

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct CurrentLetter(pub Letter);

#[derive(Resource, Debug)]
pub struct LetterBag {
    pub flavors: ShuffleBag<Handle<Flavor>>,
    pub blessings: ShuffleBag<Handle<TestimonialStub>>,
    pub curses: ShuffleBag<Handle<TestimonialStub>>,
    pub names: ShuffleBag<Handle<Name>>,
}

impl LetterBag {
    pub fn pull_letter<R: Rng>(
        &mut self,
        testimonials: &Res<Assets<TestimonialStub>>,
        flavors: Res<Assets<Flavor>>,
        names: Res<Assets<Name>>,
        rng: &mut R,
        blessing_amount: usize,
        curse_amount: usize,
    ) -> Letter {
        let flavor_handle = self.flavors.pick(rng);
        let flavor = flavors
            .get(flavor_handle)
            .expect("flavor asset must be present")
            .to_owned();

        let recipients = blessing_amount;

        let time_limit = TIME_LIMIT_RANGE
            .choose(rng)
            .expect("TIME_LIMIT_RANGE must be a valid range");

        let body = flavor
            .body
            .replace("{recipients}", &recipients.to_string())
            .replace("{time_limit}", &time_limit.to_string());

        let signoff = flavor
            .signoff
            .replace("{recipients}", &recipients.to_string())
            .replace("{time_limit}", &time_limit.to_string());

        let interpolated_flavor = InterpolatedFlavor { body, signoff };

        let blessing_handles: Vec<_> = (0..blessing_amount)
            .map(|_| self.blessings.pick(rng).clone())
            .collect();
        let blessings: Vec<Testimonial> = blessing_handles
            .iter()
            .map(|h| self.create_testimonial(h, testimonials, &names, rng))
            .collect();

        let curse_handles: Vec<_> = (0..curse_amount)
            .map(|_| self.curses.pick(rng).clone())
            .collect();
        let curses: Vec<Testimonial> = curse_handles
            .iter()
            .map(|h| self.create_testimonial(h, testimonials, &names, rng))
            .collect();

        Letter {
            flavor,
            interpolated_flavor,
            recipients,
            time_limit,
            blessings,
            curses,
        }
    }

    fn create_testimonial<R: Rng>(
        &mut self,
        handle: &Handle<TestimonialStub>,
        testimonials: &Res<Assets<TestimonialStub>>,
        names: &Res<Assets<Name>>,
        rng: &mut R,
    ) -> Testimonial {
        let name_handle = self.names.pick(rng).clone();
        let testimonial = testimonials
            .get(handle)
            .expect("testimonial asset must be present")
            .to_owned();
        let name = names
            .get(&name_handle)
            .expect("name asset must be present")
            .to_owned();
        let effect = testimonial.effect;
        let first_name = name.first_name;
        let pronouns = name.pronouns;
        let last_initial = random_initial(rng);
        let message = testimonial
            .message
            .replace("{pronoun_subject}", &pronouns.subject())
            .replace("{pronoun_object}", &pronouns.object())
            .replace("{pronoun_possessive}", &pronouns.possessive());

        // TODO: target selection.
        let target_word = get_word_at_index(&message, testimonial.targets[0])
            .expect("target word index must be valid");
        let message = replace_word_with_underscores(message, testimonial.targets[0]);
        let message = format!("{} {}. {}.", first_name, last_initial, message);

        let collected = false;

        Testimonial {
            message,
            effect,
            target_word,
            first_name,
            last_initial,
            pronouns,
            collected,
        }
    }
}

#[derive(Resource, Deref, DerefMut, Debug)]
pub struct WordBag(ShuffleBag<WordCube>);

impl WordBag {
    pub fn new<R: Rng>(
        blessings: &[Testimonial],
        curses: &[Testimonial],
        decoys: &[&str],
        rng: &mut R,
    ) -> Self {
        let blessing_cubes: Vec<_> = blessings
            .iter()
            .map(|t| {
                let color = Color::hsl(((rng.next_u32() % 180 + 165) % 360) as f32, 0.3, 0.4);
                WordCube::new(&t.target_word, color, '+')
            })
            .collect();

        let curse_cubes: Vec<_> = curses
            .iter()
            .map(|t| {
                let color = Color::hsl(((rng.next_u32() % 180 + 165) % 360) as f32, 0.3, 0.4);
                WordCube::new(&t.target_word, color, 'x')
            })
            .collect();

        let decoy_cubes: Vec<_> = decoys
            .iter()
            .map(|word| {
                let color = Color::hsl(((rng.next_u32() % 180 + 165) % 360) as f32, 0.3, 0.4);
                WordCube::new(word, color, '~')
            })
            .collect();

        let bag = ShuffleBag::try_new(
            blessing_cubes
                .into_iter()
                .chain(curse_cubes)
                .chain(decoy_cubes)
                .collect::<Vec<_>>(),
            rng,
        )
        .expect("there must be at least one testimonial");

        Self(bag)
    }
}

fn create_letter_bag_system(
    mut commands: Commands,
    letter_handles: Res<LetterAssets>,
    mut rng: Local<RngResource>,
) {
    commands.insert_resource(LetterBag {
        flavors: ShuffleBag::try_new(letter_handles.flavors.clone(), &mut rng.0)
            .expect("flavor handle list should not be empty"),
        blessings: ShuffleBag::try_new(letter_handles.blessings.clone(), &mut rng.0)
            .expect("blessing handle list should not be empty"),
        curses: ShuffleBag::try_new(letter_handles.curses.clone(), &mut rng.0)
            .expect("curse handle list should not be empty"),
        names: ShuffleBag::try_new(letter_handles.names.clone(), &mut rng.0)
            .expect("name handle list should not be empty"),
    });
}

fn random_initial<R: Rng>(rng: &mut R) -> char {
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .nth(rng.random_range(0..26))
        .unwrap()
}

fn get_word_at_index(text: &str, word_index: usize) -> Option<String> {
    text.split_whitespace()
        .nth(word_index)
        .map(ToOwned::to_owned)
}

fn replace_word_with_underscores(text: String, word_index: usize) -> String {
    text.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            if i == word_index {
                "_".repeat(word.len())
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
