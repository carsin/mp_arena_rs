mod player;
mod networking;
mod player_controller;
mod camera_controller;

use bevy::{
    prelude::*,
    window::{close_on_esc, PresentMode},
};
use player::PlayerBundleFactory;
use player_controller::{PlayerController, PlayerControllerPlugin};
use camera_controller::{CameraController, CameraControllerPlugin};
use clap::{Parser, arg};
use bevy_ggrs::prelude::*;
use ggrs::{P2PSession, PlayerType, SessionBuilder, UdpNonBlockingSocket};
use std::net::SocketAddr;

const FPS: usize = 60;
type Config = bevy_ggrs::GgrsConfig<u8>;

#[derive(Parser, Resource)]
struct Args {
    #[arg(short, long)]
    local_port: u16,
    #[arg(short, long, num_args = 1..)]
    players: Vec<String>,
    #[arg(short, long, num_args = 1..)]
    spectators: Vec<SocketAddr>,
}

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    Paused,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = Args::parse();
    let num_players = arg.players.len();
    assert!(num_players > 0);

    // create a GGRS session builder
    let mut sess_build = SessionBuilder::<Config>::new()
        .with_num_players(num_players)
        .with_desync_detection_mode(ggrs::DesyncDetection::On { interval: 10 })
        .with_input_delay(2)
        .with_max_prediction_window(12);
    
    for (i, player_addr) in opt.players.iter().enumerate() {
        if player_addr == "localhost" {
            sess_build = sess_build.add_player(PlayerType::Local, i)?;
        } else {
            let remote_addr: SocketAddr = player_addr.parse()?;
            sess_build = sess_build.add_player(PlayerType::Remote(remote_addr), i)?;
        }
    }

    let socket = UdpNonBlockingSocket::bind_to_port(arg.local_port).unwrap();
    let sess = sess_build.start_p2p_session(socket).unwrap();
    
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
            GgrsPlugin::<Config>::default()
            .set_rollback_schedule_fps(FPS),
            PlayerControllerPlugin,
            CameraControllerPlugin,
        ))
        .add_state::<GameState>()
        .insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.05)))
        .insert_resource(SessionType::new(sess))
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

    Ok(())
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
