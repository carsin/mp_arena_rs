use bevy::{
    log::LogPlugin, prelude::*, sprite::Mesh2dHandle, utils::HashMap, window::close_on_esc,
};
use bevy_renet::{
    client_connected,
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            NetcodeTransportError, ServerAuthentication, ServerConfig,
        },
        ChannelConfig, ClientId, ConnectionConfig, DefaultChannel, RenetClient, RenetServer,
        SendType, ServerEvent,
    },
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    env::consts,
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime},
};

use crate::remoteplayer::{RemotePlayer, RemotePlayerBundle, RemotePlayerBundleFactory};
use crate::GameState;

const PLAYER_SPEED: f32 = 50.;

#[derive(Debug, Serialize, Deserialize, Component, Event)]
pub enum ClientChannel {
    Input,
}
pub enum ServerChannel {
    ServerMessages,
    PlayerData,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Input => 0,
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![ChannelConfig {
            channel_id: Self::Input.into(),
            max_memory_usage_bytes: 5 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::ZERO,
            },
        }]
    }
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::ServerMessages => 0,
            ServerChannel::PlayerData => 1,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::ServerMessages.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
            ChannelConfig {
                channel_id: Self::PlayerData.into(),
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
        ]
    }
}

pub fn make_connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        server_channels_config: ServerChannel::channels_config(),
    }
}

#[derive(Debug, Default, Resource, Serialize, Deserialize)]
pub struct Lobby {
    pub player_data: HashMap<ClientId, PlayerState>,
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessage {
    PlayerConnected {
        client_id: ClientId,
        player_state: PlayerState,
    },
    PlayerDisconnected {
        client_id: ClientId,
    },
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Component)]
pub struct PlayerState {
    pub input_dir: Vec2,
    pub position: Vec3,
}

pub fn run_server(port: u16, connection_config: ConnectionConfig) {
    let server_addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 64,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    let socket = UdpSocket::bind(server_addr).unwrap();
    println!("Started server on {:?}", socket.local_addr());

    App::new()
        .add_plugins((
            MinimalPlugins,
            LogPlugin::default(),
            TransformPlugin,
            HierarchyPlugin,
            RenetServerPlugin,
            NetcodeServerPlugin,
        ))
        .add_state::<GameState>()
        .insert_resource(Lobby::default())
        .insert_resource(RenetServer::new(connection_config))
        .insert_resource(NetcodeServerTransport::new(server_config, socket).unwrap())
        .add_systems(
            Update,
            (
                server_broadcast,
                server_receive,
                server_handle_network_events,
                server_update_players,
            ),
        )
        .run();
}

fn server_handle_network_events(
    mut events: EventReader<ServerEvent>,
    mut lobby: ResMut<Lobby>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.read() {
        info!("received event: {:?}", event);

        // handle events
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);
                lobby.player_data.insert(*client_id, PlayerState::default());

                // send the current game state to the newly connected client
                let initial_state_message = bincode::serialize(&lobby.player_data).unwrap();
                server.send_message(*client_id, ServerChannel::PlayerData, initial_state_message);

                let mut rng = rand::thread_rng();
                let player_state = PlayerState {
                    input_dir: Vec2::ZERO,
                    position: Vec3::new(rng.gen::<f32>() * 10., rng.gen::<f32>() * 10., 2.),
                };

                // broadcast a message to inform other clients of the new player
                let new_player_message = bincode::serialize(&ServerMessage::PlayerConnected {
                    client_id: *client_id,
                    player_state,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, new_player_message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);
                lobby.player_data.remove(client_id);

                // broadcast player disconnection
                let disconnect_message = bincode::serialize(&ServerMessage::PlayerDisconnected {
                    client_id: *client_id,
                })
                .unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, disconnect_message);
            }
        }
    }
}

fn server_receive(mut server: ResMut<RenetServer>, mut lobby: ResMut<Lobby>) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let input_dir: Vec2 = bincode::deserialize(&message).unwrap();
            if let Some(player_state) = lobby.player_data.get_mut(&client_id) {
                player_state.input_dir = input_dir;
            } else {
                lobby.player_data.insert(
                    client_id,
                    PlayerState {
                        input_dir,
                        position: Vec3::ZERO,
                    },
                );
            }
        }
    }
}

fn server_broadcast(mut server: ResMut<RenetServer>, lobby: Res<Lobby>) {
    let message = bincode::serialize(&lobby.player_data).unwrap();
    server.broadcast_message(ServerChannel::PlayerData, message);
}

fn server_update_players(time: Res<Time>, mut lobby: ResMut<Lobby>) {
    for (client_id, player_state) in lobby.player_data.iter_mut() {
        player_state.position +=
            player_state.input_dir.extend(0.0) * PLAYER_SPEED * time.delta_seconds();
    }
}
