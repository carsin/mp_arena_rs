mod camera_controller;
mod input;
mod player;

use bevy::{prelude::*, window::close_on_esc};
use bevy_ggrs::prelude::*;
use camera_controller::{CameraController, CameraControllerPlugin};
use clap::{arg, Parser};
use ggrs::{P2PSession, PlayerType, SessionBuilder, UdpNonBlockingSocket};
use input::PlayerInput;
use player::PlayerBundleFactory;
use std::net::SocketAddr;

const FPS: usize = 60;
type Config = bevy_ggrs::GgrsConfig<PlayerInput>;

#[derive(Parser, Resource)]
struct Args {
    #[arg(short, long)]
    port: u16,

    #[arg(short, long, num_args = 1..)]
    remote_players: Vec<SocketAddr>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    Paused,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let player_count = args.remote_players.len() + 1;

    let mut session_builder = SessionBuilder::<Config>::new()
        .with_num_players(player_count)
        .with_desync_detection_mode(ggrs::DesyncDetection::On { interval: 10 });

    // Add local player (always handle 0)
    session_builder = session_builder.add_player(PlayerType::Local, 0)?;

    // Add other players (handles 1+)
    for (i, player_addr) in args.remote_players.iter().enumerate() {
        let handle = i + 1;
        session_builder =
            session_builder.add_player(PlayerType::Remote(player_addr.clone()), handle)?;
    }

    let socket = UdpNonBlockingSocket::bind_to_port(args.port).unwrap();
    let session = session_builder.start_p2p_session(socket).unwrap();

    start_app(session);

    Ok(())
}

fn start_app(session: P2PSession<Config>) {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,            // always fill entire window
                    prevent_default_event_handling: false, // don't hijack keyboard shortcuts
                    ..default()
                }),
                ..default()
            }),
            GgrsPlugin::<Config>::default(),
            CameraControllerPlugin,
        ))
        .set_rollback_schedule_fps(FPS)
        .rollback_component_with_clone::<Transform>()
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.05)))
        .insert_resource(Session::P2P(session))
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
        .add_rollback()
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

fn read_local_input() {}
