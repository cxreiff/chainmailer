use std::f32::consts::PI;

use bevy::prelude::*;
use rand::RngCore;
use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Style, Stylize},
    widgets::WidgetRef,
};

use crate::{constants::CONFETTI_AMOUNT, rng::RngResource, states::GameStates};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(confetti_spawn_observer).add_systems(
        Update,
        (
            move_confetti_system.run_if(in_state(GameStates::Playing)),
            move_confetti_system.run_if(in_state(GameStates::Resetting)),
            despawn_confetti_system,
        ),
    );
}

#[derive(Component, Debug)]
pub struct Confetti {
    pub color: ratatui::style::Color,
    pub character: char,
    pub position: Vec3,
    pub velocity: Vec3,
    pub timer: Timer,
}

#[derive(Debug)]
pub struct ConfettiWidget<'a> {
    pub confetti: &'a Confetti,
    pub cell: IVec2,
}

impl<'a> ConfettiWidget<'a> {
    pub fn new(confetti: &'a Confetti, cell: IVec2) -> Self {
        Self { confetti, cell }
    }
}

impl WidgetRef for ConfettiWidget<'_> {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let position = Position::from((self.cell.x as u16, self.cell.y as u16));
        if area.contains(position) {
            buf.cell_mut(position)
                .unwrap()
                .set_char(self.confetti.character)
                .set_fg(self.confetti.color)
                .set_style(Style::default().bold());
        }
    }
}

#[derive(Event, Debug)]
pub struct ConfettiSpawn {
    pub position: Vec3,
    pub character: char,
    pub color: ratatui::style::Color,
}

fn confetti_spawn_observer(
    confetti_spawn: On<ConfettiSpawn>,
    mut commands: Commands,
    mut rng: Local<RngResource>,
) {
    for index in 0..CONFETTI_AMOUNT {
        let theta = (2.0 * PI / CONFETTI_AMOUNT as f32) * index as f32;
        let x = confetti_spawn.position.x + theta.cos() * 0.02;
        let y = confetti_spawn.position.y + theta.sin() * 0.02;
        let z = confetti_spawn.position.z
            + (rng.next_u32() as f64 / u32::MAX as f64 - 0.5) as f32 * 0.3;

        let position = Vec3::new(x, y, z);
        let velocity = (position - confetti_spawn.position).normalize() * 0.01;

        commands.spawn(Confetti {
            position,
            velocity,
            character: confetti_spawn.character,
            color: confetti_spawn.color,
            timer: Timer::from_seconds(2.0, TimerMode::Once),
        });
    }
}

fn move_confetti_system(mut confettis: Query<&mut Confetti>) {
    for mut confetti in &mut confettis {
        let confetti_velocity = confetti.velocity;
        confetti.position += confetti_velocity;
        confetti.velocity.y -= 0.0004;
        confetti.velocity.x -= confetti.velocity.x * 0.0001;
        confetti.velocity.z -= confetti.velocity.z * 0.0001;
    }
}

fn despawn_confetti_system(
    mut commands: Commands,
    time: Res<Time>,
    mut confettis: Query<(Entity, &mut Confetti)>,
) {
    for (entity, mut confetti) in &mut confettis {
        confetti.timer.tick(time.delta());

        if confetti.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
