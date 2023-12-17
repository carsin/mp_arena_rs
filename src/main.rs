mod camera_controller;
mod player;
mod player_controller;

use bevy::{
    prelude::*,
    window::{close_on_esc, PresentMode},
};
use player::PlayerBundleFactory;
use player_controller::{PlayerController, PlayerControllerPlugin};
use camera_controller::{CameraController, CameraControllerPlugin};

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    Paused,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true, // always fill entire window
                    prevent_default_event_handling: false, // don't hijack keyboard shortcuts 
                    ..default()
                }),
                ..default()
            }),
            PlayerControllerPlugin,
            CameraControllerPlugin,
        ))
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.05)))
        .add_systems(
            Startup,
            (
                spawn_camera,
                // Make sure camera entity is added to world before spawning
                // player, which needs to access it
                apply_deferred.after(spawn_camera).before(spawn_player),
                spawn_player,
                test_spawn_dummy_player,
            ),
        )
        .add_systems(Update, close_on_esc)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.05;

    commands.spawn((camera_bundle, CameraController::default()));
}

fn spawn_player(
    mut commands: Commands,
    mut player_factory: PlayerBundleFactory,
    mut cameras: Query<&mut CameraController>,
) {
    let entity = commands
        .spawn(player_factory.build())
        .insert(PlayerController::default())
        .id();

    for mut camera_controller in cameras.iter_mut() {
        camera_controller.target = Some(entity);
    }
}

/// TEST: spawns a player with no player controller
fn test_spawn_dummy_player(mut commands: Commands, mut player_factory: PlayerBundleFactory) {
    for _ in 0..15 {
        let mut dummy = player_factory.build();
        dummy.mesh.transform = Transform::from_translation(
            (Vec3::new(rand::random(), rand::random(), rand::random()) * 2.0 - 1.0) * 30.0,
        )
        .with_rotation(Quat::from_rotation_z(
            rand::random::<f32>() * std::f32::consts::PI * 2.0,
        ));

        commands.spawn(dummy);
    }
}
