use bevy::{log::LogPlugin, prelude::*, utils::hashbrown::HashMap};
use bevy_renet::{
    renet::{
        transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
        ClientId, ConnectionConfig, RenetServer, ServerEvent,
    },
    transport::NetcodeServerPlugin,
    RenetServerPlugin,
};
use renet::DefaultChannel;
use std::{net::UdpSocket, time::SystemTime};

use crate::{
    messages::{ClientMessage, ServerMessage},
    player_controller::{PlayerController, PlayerControllerPlugin},
};
use crate::{remoteplayer_controller::RemotePlayerController, GameState};

pub fn make_connection_config() -> ConnectionConfig {
    ConnectionConfig::default()
    // ConnectionConfig {
    //     available_bytes_per_tick: 1024 * 1024,
    //     client_channels_config: ClientChannel::channels_config(),
    //     server_channels_config: ServerChannel::channels_config(),
    // }
}

// Maps client IDs to player entities.
#[derive(Deref, DerefMut, Resource, Default)]
pub struct ClientMap(HashMap<ClientId, Entity>);

// Maps player entities to client IDs. Attached as a component to player
// entities.
#[derive(Component, Deref, DerefMut)]
pub struct PlayerClient(ClientId);

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
            PlayerControllerPlugin { headless: true },
        ))
        .add_state::<GameState>()
        .insert_resource(ClientMap::default())
        .insert_resource(RenetServer::new(connection_config))
        .insert_resource(NetcodeServerTransport::new(server_config, socket).unwrap())
        .add_systems(
            Update,
            (
                server_receive,
                server_handle_network_events,
                server_broadcast,
            ),
        )
        .run();
}

fn server_handle_network_events(
    mut commands: Commands,
    mut client_map: ResMut<ClientMap>,
    mut events: EventReader<ServerEvent>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.read() {
        // handle events
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Player {} connected.", client_id);

                // Spawn the player
                let player_entity = commands
                    .spawn((
                        PlayerClient(*client_id),
                        PlayerController::default(),
                        TransformBundle::default(),
                    ))
                    .id();
                client_map.insert(*client_id, player_entity);

                // broadcast a message to inform other clients of the new player
                let new_player_message = bincode::serialize(&ServerMessage::PlayerConnected {
                    client_id: *client_id,
                })
                .unwrap();
                server.broadcast_message(DefaultChannel::ReliableOrdered, new_player_message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Player {} disconnected: {}", client_id, reason);

                // broadcast player disconnection
                let disconnect_message = bincode::serialize(&ServerMessage::PlayerDisconnected {
                    client_id: *client_id,
                })
                .unwrap();
                server.broadcast_message(DefaultChannel::ReliableOrdered, disconnect_message);
            }
        }
    }
}

fn server_receive(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    client_map: Res<ClientMap>,
) {
    for client_id in server.clients_id() {
        while let Some(bytes) = server.receive_message(client_id, DefaultChannel::ReliableOrdered) {
            // Retrieve player entity and commands for this client
            //
            // TODO: these would be better placed outside the above while loop,
            // but this causes the values to be looked up for every client even
            // when they have not sent any messages.
            let Some(player_entity) = client_map.get(&client_id) else {
                warn!(
                    "Received controls from client that has no mapped ID (client ID: {})",
                    client_id
                );
                continue;
            };
            let Some(mut player_commands) = commands.get_entity(*player_entity) else {
                warn!(
                    "Received controls from client whose mapped entity is missing (client ID: {})",
                    client_id
                );
                continue;
            };

            // Update the player controller from client
            let msg: ClientMessage = match bincode::deserialize(&bytes) {
                Ok(msg) => msg,
                Err(err) => {
                    warn!("Failed to deserialize client message: {}", err);
                    continue;
                }
            };
            match msg {
                ClientMessage::Controller(controller) => {
                    player_commands.insert(controller);
                }
            }
        }
    }
}

fn server_broadcast(
    mut server: ResMut<RenetServer>,
    players: Query<(&Transform, &PlayerClient), With<PlayerController>>,
) {
    let msg = ServerMessage::Players(
        players
            .iter()
            .map(|(transform, player_client)| {
                (
                    **player_client,
                    RemotePlayerController {
                        server_position: transform.translation.xy(),
                        server_angle: transform.rotation.to_euler(EulerRot::XYZ).2,
                    },
                )
            })
            .collect(),
    );
    let bytes = match bincode::serialize(&msg) {
        Ok(msg) => msg,
        Err(err) => {
            warn!("Failed to serlialize players message: {}", err);
            return;
        }
    };
    server.broadcast_message(DefaultChannel::ReliableOrdered, bytes);
}
