use bevy::{prelude::*, utils::HashMap, window::close_on_esc};
use bevy_renet::{
    renet::{transport::ClientAuthentication, ClientId, ConnectionConfig, RenetClient},
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};
use renet::{transport::NetcodeClientTransport, DefaultChannel};
use serde::{Deserialize, Serialize};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use crate::{
    camera_controller::{CameraController, CameraControllerPlugin},
    messages::ServerMessage,
};
use crate::{messages::ClientMessage, rendering::RendererPlugin};
use crate::{
    player_controller::{PlayerController, PlayerControllerPlugin},
    remote_state::RemotePlayerControllerPlugin,
};
use crate::{rendering::PlayerRendererBundleFactory, GameState};

#[derive(Debug, Default, Serialize, Deserialize, Component, Clone)]
struct PlayerInput {
    direction: Vec2,
}

#[derive(Debug, Default, Resource, Deref, DerefMut)]
struct ClientMap(HashMap<ClientId, Entity>);

#[derive(Debug, Resource, Deref, DerefMut)]
struct LocalClientId(u64);

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
            PlayerControllerPlugin { headless: false },
            RemotePlayerControllerPlugin,
            CameraControllerPlugin,
            RenetClientPlugin,
            NetcodeClientPlugin,
            RendererPlugin,
        ))
        .add_state::<GameState>()
        .insert_resource(ClientMap::default())
        .insert_resource(ClearColor(Color::hsl(0.0, 0.0, 0.05)))
        .insert_resource(RenetClient::new(connection_config))
        .insert_resource(LocalClientId(client_id))
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
        .add_systems(Startup, (spawn_camera, spawn_players, spawn_local_player))
        .add_systems(Update, (client_send_input, client_receive))
        .add_systems(Update, close_on_esc)
        .run();
}

fn client_send_input(mut client: ResMut<RenetClient>, controllers: Query<&PlayerController>) {
    let message =
        bincode::serialize(&ClientMessage::Controller(controllers.single().clone())).unwrap();
    client.send_message(DefaultChannel::ReliableOrdered, message);
}

fn client_receive(
    mut commands: Commands,
    mut player_factory: PlayerRendererBundleFactory,
    mut client: ResMut<RenetClient>,
    mut client_map: ResMut<ClientMap>,
    local_client_id: Res<LocalClientId>,
) {
    while let Some(msg) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let msg: ServerMessage = bincode::deserialize(&msg).unwrap();

        match msg {
            ServerMessage::PlayerConnected { client_id } => {
                info!("Player {} connected.", client_id);
            }
            ServerMessage::PlayerDisconnected { client_id } => {
                info!("Player {} disconnected.", client_id);
            }
            ServerMessage::Players(players) => {
                for (client_id, controller) in players {
                    // if client_id.raw() == **local_client_id {
                    //     continue;
                    // }

                    if let Some(player_entity) = client_map.get_mut(&client_id) {
                        commands.entity(*player_entity).insert(controller);
                    } else {
                        // Spawn player
                        let player_entity = commands
                            .spawn((controller, TransformBundle::default()))
                            .id();

                        // Spawn player renderer
                        commands.spawn(player_factory.build(player_entity));

                        // Register client ID -> player mapping
                        client_map.insert(client_id, player_entity);
                    }
                }
            }
            ServerMessage::Bullets(bullets) => {}
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.05;

    commands.spawn((camera_bundle, CameraController::default()));
}

fn spawn_local_player(mut commands: Commands, mut player_factory: PlayerRendererBundleFactory) {
    commands.spawn((PlayerController::default(), TransformBundle::default()));
}

fn spawn_players(
    mut commands: Commands,
    mut player_factory: PlayerRendererBundleFactory,
    mut cameras: Query<&mut CameraController>,
) {
    // let entity = commands
    //     .spawn(player_factory.build())
    //     .insert(PlayerController::default())
    //     .id();

    // for mut camera_controller in cameras.iter_mut() {
    //     camera_controller.target = Some(entity);
    // }
}
