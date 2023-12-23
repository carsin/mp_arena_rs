use bevy::{
    log::LogPlugin, prelude::*, sprite::Mesh2dHandle, utils::HashMap, utils::HashSet,
    window::close_on_esc,
};
use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, ServerAuthentication, ServerConfig},
        ClientId, ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent,
    },
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use clap::Parser;
use renet::transport::{NetcodeClientTransport, NetcodeServerTransport, NetcodeTransportError};
use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use crate::camera_controller::{CameraController, CameraControllerPlugin};
use crate::player::PlayerBundleFactory;
use crate::player_controller::{PlayerController, PlayerControllerPlugin};
use crate::server::{ServerChannel, ClientChannel, Lobby, PlayerState, ServerMessage};
use crate::GameState;

#[derive(Component)]
struct PlayerEntity {
    client_id: ClientId,
}

#[derive(Debug, Default, Serialize, Deserialize, Component, Clone)]
struct PlayerInput {
    direction: Vec2,
}

pub fn run_client(server_address: SocketAddr, connection_config: ConnectionConfig) {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;

    let authentication = ClientAuthentication::Unsecure {
        protocol_id: 0,
        client_id,
        server_addr: server_address,
        user_data: None,
    };

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins,
            PlayerControllerPlugin,
            CameraControllerPlugin,
            RenetClientPlugin,
            NetcodeClientPlugin,
        ))
        .add_state::<GameState>()
        .insert_resource(Lobby::default())
        .insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.05)))
        .insert_resource(RenetClient::new(connection_config))
        .insert_resource(
            NetcodeClientTransport::new(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap(),
                authentication,
                socket,
            )
            .unwrap(),
        )
        .add_systems(
            Startup,
            (
                spawn_camera,
                // Make sure camera entity is added to world before spawning
                // player, which needs to access it
                apply_deferred.after(spawn_camera).before(spawn_player),
                spawn_player,
            ),
        )
        .add_systems(
            Update,
            (client_send_input, client_receive, update_player_entities),
        )
        .add_systems(Update, close_on_esc)
        .run();
}

fn client_send_input(
    mut client: ResMut<RenetClient>,
    query: Query<(&PlayerController, &Transform), With<PlayerEntity>>,
) {
    if let Some((controller, transform)) = query.iter().next() {
        let player_state = PlayerState {
            input_dir: controller.move_direction,
            position: transform.translation,
        };

        let message = bincode::serialize(&player_state.input_dir).unwrap();
        client.send_message(ClientChannel::Input, message);
    }
}

fn client_receive(mut client: ResMut<RenetClient>, mut lobby: ResMut<Lobby>) {
    while let Some(message) = client.receive_message(ServerChannel::PlayerData) {
        // info!("received data message: {:?}", message);

        let players: HashMap<ClientId, PlayerState> = bincode::deserialize(&message).unwrap();
        lobby.player_data = players;
    }

    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message: ServerMessage = bincode::deserialize(&message).unwrap();
        info!("received server message: {:?}", server_message);
        match server_message {
            _ => (), 
        }
    }
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

fn update_player_entities(
    mut commands: Commands,
    lobby: Res<Lobby>,
    mut query: Query<(Entity, &mut Transform, &PlayerEntity)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let existing_players = &lobby.player_data;

    // Update existing players and mark them as seen
    for (entity, mut transform, player_entity) in query.iter_mut() {
        if let Some(player_state) = lobby.player_data.get(&player_entity.client_id) {
            transform.translation = player_state.position;
        } else {
            // Despawn entities that are no longer in the game state
            commands.entity(entity).despawn();
        }
    }

    // Spawn entities for new players
    for (&client_id, player_state) in lobby.player_data.iter() {
        if !existing_players.contains_key(&client_id) {
            commands
                .spawn(ColorMesh2dBundle {
                    mesh: Mesh2dHandle(
                        meshes.add(
                            shape::Quad {
                                size: Vec2::splat(1.2),
                                ..Default::default()
                            }
                            .into(),
                        ),
                    ),
                    material: materials.add(ColorMaterial {
                        color: Color::YELLOW,
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(player_state.position),
                    ..Default::default()
                })
                .insert(PlayerEntity { client_id });
        }
    }
}