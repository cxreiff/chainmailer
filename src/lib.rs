use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_ratatui::RatatuiPlugins;
use bevy_ratatui_camera::RatatuiCameraPlugin;

mod constants;
mod input;
mod interface;
mod letters;
mod loading;
mod rng;
mod scene;
mod sound;
mod states;
#[cfg(not(feature = "windowed"))]
mod terminal;
#[cfg(feature = "windowed")]
mod windowed;
mod word_checks;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            #[cfg(not(feature = "windowed"))]
            terminal::plugin,
            #[cfg(feature = "windowed")]
            windowed::plugin,
        ));

        app.add_plugins((
            FrameTimeDiagnosticsPlugin {
                smoothing_factor: 1.0,
                ..default()
            },
            EntityCountDiagnosticsPlugin::default(),
            RatatuiPlugins {
                enable_mouse_capture: true,
                ..default()
            },
            RatatuiCameraPlugin,
        ));

        app.add_plugins((
            interface::plugin,
            input::plugin,
            letters::plugin,
            loading::plugin,
            sound::plugin,
            scene::plugin,
            states::plugin,
            word_checks::plugin,
        ));

        app.insert_resource(ClearColor(Color::srgba(0., 0., 0., 0.)));
    }
}
