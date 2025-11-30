use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use rand::seq::IndexedRandom;

use crate::{interface::draw::Flags, rng::RngResource};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(sound_effects_observer);
}

#[derive(AssetCollection, Resource)]
pub struct SoundEffectAssets {
    #[asset(key = "sounds.window")]
    pub window: Handle<AudioSource>,
    #[asset(key = "sounds.text_group")]
    pub text_group: Handle<AudioSource>,
    #[asset(key = "sounds.text_character", collection(typed))]
    pub text_character: Vec<Handle<AudioSource>>,
    #[asset(key = "sounds.text_header_bless")]
    pub text_header_bless: Handle<AudioSource>,
    #[asset(key = "sounds.text_bless")]
    pub text_bless: Handle<AudioSource>,
    #[asset(key = "sounds.text_header_curse")]
    pub text_header_curse: Handle<AudioSource>,
    #[asset(key = "sounds.text_curse")]
    pub text_curse: Handle<AudioSource>,
    #[asset(key = "sounds.start")]
    pub start: Handle<AudioSource>,
    #[asset(key = "sounds.letter_clear")]
    pub letter_clear: Handle<AudioSource>,
    #[asset(key = "sounds.letter_fail")]
    pub letter_fail: Handle<AudioSource>,
    #[asset(key = "sounds.guess_bless")]
    pub guess_bless: Handle<AudioSource>,
    #[asset(key = "sounds.guess_decoy")]
    pub guess_decoy: Handle<AudioSource>,
    #[asset(key = "sounds.guess_curse")]
    pub guess_curse: Handle<AudioSource>,
}

#[derive(Event)]
pub enum SoundEffect {
    Window,
    TextGroup,
    TextCharacter,
    TextHeaderBless,
    TextBless,
    TextHeaderCurse,
    TextCurse,
    Start,
    LetterClear,
    LetterFail,
    GuessBless,
    GuessDecoy,
    GuessCurse,
}

fn sound_effects_observer(
    sound_effect: On<SoundEffect>,
    mut commands: Commands,
    handles: Res<SoundEffectAssets>,
    mut rng: Local<RngResource>,
    flags: Res<Flags>,
) {
    if !flags.sound {
        return;
    }

    let sound = match *sound_effect {
        SoundEffect::Window => &handles.window,
        SoundEffect::TextGroup => &handles.text_group,
        SoundEffect::TextCharacter => handles.text_character.choose(&mut rng.0).unwrap(),
        SoundEffect::TextHeaderBless => &handles.text_header_bless,
        SoundEffect::TextBless => &handles.text_bless,
        SoundEffect::TextHeaderCurse => &handles.text_header_curse,
        SoundEffect::TextCurse => &handles.text_curse,
        SoundEffect::Start => &handles.start,
        SoundEffect::LetterClear => &handles.letter_clear,
        SoundEffect::LetterFail => &handles.letter_fail,
        SoundEffect::GuessBless => &handles.guess_bless,
        SoundEffect::GuessDecoy => &handles.guess_decoy,
        SoundEffect::GuessCurse => &handles.guess_curse,
    };

    commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
}
