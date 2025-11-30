use crate::interface::widgets::prompt::Prompt;
use crate::states::GameStates;
#[cfg(feature = "windowed")]
use bevy::input::ButtonState;
#[cfg(feature = "windowed")]
use bevy::input::keyboard::KeyboardInput;
#[cfg(feature = "windowed")]
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
#[cfg(not(feature = "windowed"))]
use bevy_ratatui::event::KeyMessage as RatatuiKeyMessage;
#[cfg(not(feature = "windowed"))]
use bevy_ratatui::event::MouseMessage as RatatuiMouseMessage;

use crate::interface::draw::Flags;
use crate::interface::widgets::letter::LetterWidgetState;
use crate::sound::SoundEffect;
use crate::word_checks::SubmittedWord;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            handle_keyboard_input_system,
            handle_mouse_input_system,
            handle_prompt_input_system,
            pass_info_screen_system.run_if(in_state(GameStates::Info)),
        ),
    );
}

#[cfg(not(feature = "windowed"))]
fn pass_info_screen_system(
    mut commands: Commands,
    mut keyboard_input: MessageReader<RatatuiKeyMessage>,
) {
    use bevy_ratatui::crossterm::event::KeyCode;

    for message in keyboard_input.read() {
        if message.code == KeyCode::Char(' ') {
            commands.set_state(GameStates::Printing);
        }
    }
}

#[cfg(feature = "windowed")]
fn pass_info_screen_system(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    for &press in keyboard_input.get_just_pressed() {
        if press == KeyCode::Space {
            commands.set_state(GameStates::Printing);
        };
    }
}

#[cfg(not(feature = "windowed"))]
fn handle_keyboard_input_system(
    mut commands: Commands,
    mut keyboard_input: MessageReader<RatatuiKeyMessage>,
    mut flags: ResMut<Flags>,
    mut current_letter_state: NonSendMut<LetterWidgetState>,
    game_state: Res<State<GameStates>>,
) {
    use bevy_ratatui::crossterm::event::KeyCode;
    use bevy_ratatui::crossterm::event::KeyEventKind;

    for message in keyboard_input.read() {
        if message.kind == KeyEventKind::Press {
            if message.code == KeyCode::Char('=') {
                flags.debug = !flags.debug;
            }
            if message.code == KeyCode::Tab {
                flags.sound = !flags.sound;
            }
            if message.code == KeyCode::Up {
                current_letter_state.scroll_state.scroll_up();
            }
            if message.code == KeyCode::Down {
                current_letter_state.scroll_state.scroll_down();
            }
            if message.code == KeyCode::Enter && *game_state == GameStates::Playing {
                commands.trigger(SubmittedWord);
            }
        }
    }
}

#[cfg(not(feature = "windowed"))]
fn handle_mouse_input_system(
    mut mouse_input: MessageReader<RatatuiMouseMessage>,
    mut letter_state: NonSendMut<LetterWidgetState>,
) {
    use ratatui::crossterm::event::MouseEventKind;

    for message in mouse_input.read() {
        match message.kind {
            MouseEventKind::ScrollUp => {
                letter_state.scroll_state.scroll_up();
            }
            MouseEventKind::ScrollDown => {
                letter_state.scroll_state.scroll_down();
            }
            _ => {}
        }
    }
}

#[cfg(feature = "windowed")]
fn handle_keyboard_input_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut flags: ResMut<Flags>,
    mut current_letter_state: NonSendMut<LetterWidgetState>,
    game_state: Res<State<GameStates>>,
) {
    for &press in keyboard_input.get_just_pressed() {
        if press == KeyCode::Tab {
            flags.sound = !flags.sound;
        };
        if press == KeyCode::ArrowUp {
            current_letter_state.scroll_state.scroll_up();
        }
        if press == KeyCode::ArrowDown {
            current_letter_state.scroll_state.scroll_down();
        }
        if press == KeyCode::Enter && *game_state == GameStates::Playing {
            commands.trigger(SubmittedWord);
        }
    }
}

#[cfg(feature = "windowed")]
fn handle_mouse_input_system(
    mut letter_state: NonSendMut<LetterWidgetState>,
    mut mouse_wheel_messages: MessageReader<MouseWheel>,
) {
    for message in mouse_wheel_messages.read() {
        if message.y > 0.0 {
            letter_state.scroll_state.scroll_up();
        } else if message.y < 0.0 {
            letter_state.scroll_state.scroll_down();
        }
    }
}

#[cfg(not(feature = "windowed"))]
fn handle_prompt_input_system(
    mut commands: Commands,
    mut keyboard_input: MessageReader<RatatuiKeyMessage>,
    mut prompt_state: ResMut<Prompt>,
) {
    use bevy_ratatui::crossterm::event::KeyCode;
    use bevy_ratatui::crossterm::event::KeyEventKind;

    for message in keyboard_input.read() {
        if message.kind == KeyEventKind::Press {
            if let KeyCode::Char(c) = message.code {
                if c.is_alphabetic() {
                    commands.trigger(SoundEffect::TextCharacter);
                    prompt_state.text.push(c.to_ascii_lowercase());
                }
            } else if let KeyCode::Backspace = message.code {
                commands.trigger(SoundEffect::TextCharacter);
                prompt_state.text.pop();
            }
        }
    }
}

#[cfg(feature = "windowed")]
fn handle_prompt_input_system(
    mut commands: Commands,
    mut keyboard_messages: MessageReader<KeyboardInput>,
    mut prompt_state: ResMut<Prompt>,
) {
    for message in keyboard_messages.read() {
        info!(
            "{:?}, {:?}, {:?}",
            message.key_code,
            KeyCode::Backspace,
            message.key_code == KeyCode::Backspace
        );
        if message.state == ButtonState::Pressed {
            if let Some(text) = &message.text {
                for c in text.chars() {
                    if c.is_alphabetic() {
                        commands.trigger(SoundEffect::TextCharacter);
                        prompt_state.text.push(c.to_ascii_lowercase());
                    }
                }
            }

            if message.key_code == KeyCode::Backspace {
                commands.trigger(SoundEffect::TextCharacter);
                prompt_state.text.pop();
            }
        }
    }
}
