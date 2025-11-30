use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use bevy_ratatui_camera::RatatuiCamera;
use rand::distr::uniform::SampleRange;

use crate::{constants::WORD_CUBE_LENGTH, letters::WordBag, rng::RngResource, states::GameStates};

pub fn plugin(app: &mut App) {
    app.add_systems(OnExit(GameStates::Loading), scene_setup_system)
        .add_systems(
            Update,
            (
                word_cube_spawn_system.run_if(on_timer(Duration::from_millis(1000))),
                word_cube_move_system,
                word_cube_despawn_system,
            )
                .run_if(in_state(GameStates::Playing)),
        );
}

#[derive(Component, Debug, Default, Clone)]
pub struct WordCube {
    pub word: String,
    pub color: Color,
    pub despawn_character: char,
}

impl WordCube {
    pub fn new(word: &str, color: Color, despawn_character: char) -> Self {
        Self {
            word: word.into(),
            color,
            despawn_character,
        }
    }
}

fn scene_setup_system(mut commands: Commands) {
    commands.spawn((PointLight::default(),));
}

fn word_cube_spawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut word_bag: ResMut<WordBag>,
    word_cubes: Query<&WordCube>,
    mut rng: Local<RngResource>,
    camera: Single<(&Camera, &GlobalTransform), With<RatatuiCamera>>,
) {
    let (camera, camera_transform) = camera.into_inner();
    let Some(spawn_position) = get_spawn_position(camera, camera_transform, 2.0, 4.0, &mut rng)
    else {
        return;
    };

    let word_cube = word_bag.pick(&mut rng.0).clone();

    if word_cubes
        .iter()
        .any(|spawned_cube| word_cube.word == spawned_cube.word)
    {
        word_bag.shuffle_new_draft(&mut rng.0);
        return;
    }

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_length(WORD_CUBE_LENGTH))),
        MeshMaterial3d(materials.add(word_cube.color)),
        Transform::from_translation(spawn_position),
        word_cube,
    ));
}

fn word_cube_move_system(time: Res<Time>, mut stars: Query<&mut Transform, With<WordCube>>) {
    for mut star in &mut stars {
        star.translation.y -= time.delta_secs() * 0.45;
        star.rotate_y(time.delta_secs());
        star.rotate_x(time.delta_secs() * 0.4);
    }
}

fn word_cube_despawn_system(
    mut commands: Commands,
    mut stars: Query<(Entity, &mut Transform), With<WordCube>>,
    camera: Single<(&Camera, &GlobalTransform), With<RatatuiCamera>>,
) {
    let (camera, camera_transform) = camera.into_inner();
    let Some(lowest_visible_y) = get_lowest_visible_y(camera, camera_transform, 4.0) else {
        return;
    };

    for (entity, transform) in &mut stars {
        if transform.translation.y < lowest_visible_y {
            commands.entity(entity).despawn();
        }
    }
}

fn get_spawn_position(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    near_depth: f32,
    far_depth: f32,
    rng: &mut RngResource,
) -> Option<Vec3> {
    let viewport_size = camera.logical_viewport_size()?;

    let top_left = camera
        .viewport_to_world(camera_transform, Vec2::new(0.05, 0.0))
        .ok()?;

    let top_right = camera
        .viewport_to_world(camera_transform, Vec2::new(viewport_size.x - 0.05, 0.0))
        .ok()?;

    let z = (near_depth..far_depth).sample_single(&mut rng.0).ok()?;

    let top_left_distance =
        top_left.intersect_plane(Vec3::new(0., 0., -z), InfinitePlane3d::new(Vec3::Z))?;
    let top_right_distance =
        top_left.intersect_plane(Vec3::new(0., 0., -z), InfinitePlane3d::new(Vec3::Z))?;

    let top_left_at_z = top_left.get_point(top_left_distance);
    let top_right_at_z = top_right.get_point(top_right_distance);

    let x = (top_left_at_z.x..top_right_at_z.x)
        .sample_single(&mut rng.0)
        .ok()?;
    let y = top_left_at_z.y + WORD_CUBE_LENGTH;

    Some(Vec3::new(x, y, -z))
}

fn get_lowest_visible_y(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    far_depth: f32,
) -> Option<f32> {
    let viewport_size = camera.logical_viewport_size()?;

    let bottom_left = camera
        .viewport_to_world(camera_transform, Vec2::new(0.0, viewport_size.y))
        .ok()?;

    let bottom_left_distance = bottom_left
        .intersect_plane(Vec3::new(0., 0., -far_depth), InfinitePlane3d::new(Vec3::Z))?;

    let bottom_left_at_depth = bottom_left.get_point(bottom_left_distance);

    Some(bottom_left_at_depth.y)
}
