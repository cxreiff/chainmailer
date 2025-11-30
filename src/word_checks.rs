use bevy::prelude::*;

use crate::{
    constants::{MAC_GREEN_COLOR, MAC_PURPLE_COLOR, MAC_RED_COLOR, MAC_YELLOW_COLOR},
    interface::widgets::{confetti::ConfettiSpawn, prompt::Prompt},
    letters::{CurrentLetter, Effect, WordBag},
    rng::RngResource,
    scene::spawning::WordCube,
    sound::SoundEffect,
    states::{LetterCleared, Statistics},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(submitted_word_observer)
        .add_observer(activate_effect_observer);
}

#[derive(Event)]
pub struct SubmittedWord;

#[derive(Event)]
pub struct ActivateEffect(Effect);

fn submitted_word_observer(
    _event: On<SubmittedWord>,
    mut commands: Commands,
    mut prompt: ResMut<Prompt>,
    mut current_letter: ResMut<CurrentLetter>,
    word_cubes: Query<(Entity, &WordCube, &Transform)>,
    mut word_bag: ResMut<WordBag>,
    mut rng: Local<RngResource>,
) {
    for (entity, word_cube, transform) in &word_cubes {
        let mut decoy = false;

        if let Some(index) = word_bag
            .full_collection
            .iter()
            .position(|word_cube| word_cube.word == prompt.text)
        {
            word_bag.full_collection.remove(index);
        };
        word_bag.reset(&mut rng.0);

        if word_cube.word == prompt.text {
            decoy = true;

            commands.entity(entity).despawn();
            commands.trigger(ConfettiSpawn {
                position: transform.translation,
                color: color_for_character(&word_cube.despawn_character),
                character: word_cube.despawn_character,
            });
        }

        for blessing in &mut current_letter.blessings {
            if blessing.target_word == prompt.text {
                blessing.collected = true;
                decoy = false;
                commands.trigger(SoundEffect::GuessBless);
                commands.trigger(ActivateEffect(blessing.effect.clone()));
            }
        }

        for curse in &mut current_letter.curses {
            if curse.target_word == prompt.text {
                curse.collected = true;
                decoy = false;
                commands.trigger(SoundEffect::GuessCurse);
                commands.trigger(ActivateEffect(curse.effect.clone()));
            }
        }

        if decoy {
            commands.trigger(SoundEffect::GuessDecoy);
        }
    }

    prompt.text = "".into();

    if current_letter
        .blessings
        .iter()
        .all(|blessing| blessing.collected)
    {
        commands.trigger(LetterCleared);
    }
}

fn color_for_character(character: &char) -> ratatui::style::Color {
    match character {
        '+' => MAC_GREEN_COLOR,
        'x' => MAC_RED_COLOR,
        '~' => MAC_YELLOW_COLOR,
        _ => MAC_PURPLE_COLOR,
    }
}

fn activate_effect_observer(activate_effect: On<ActivateEffect>, mut stats: ResMut<Statistics>) {
    match activate_effect.0 {
        Effect::Score(score) => {
            stats.score += score;
        }
        Effect::Money(money) => {
            stats.money += money;
        }
        Effect::Income(income) => stats.income += income,
        Effect::Noop => {}
    }
}
