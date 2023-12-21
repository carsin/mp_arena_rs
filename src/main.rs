mod camera_controller;
mod player;
mod player_controller;

use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{log::LogPlugin, prelude::*, window::close_on_esc};
use bevy_renet::{
    renet::{
        transport::{
            ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport,
            ServerAuthentication, ServerConfig,
        },
        ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent,
    },
    RenetClientPlugin, RenetServerPlugin, transport::{NetcodeClientPlugin, NetcodeServerPlugin},
};
use camera_controller::{CameraController, CameraControllerPlugin};
use clap::Parser;
use player::PlayerBundleFactory;
use player_controller::{PlayerController, PlayerControllerPlugin};

#[derive(Clone, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
    Paused,
}

#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Server {
        #[arg(short, long, default_value = "20987")]
        port: u16,
    },
    Client {
        #[arg(short, long, default_value = "127.0.0.1:20987")]
        server_address: SocketAddr,
    },
}

fn main() {
    let cli = Cli::parse();

    let connection_config = ConnectionConfig::default();

    match cli.subcommand {
        Subcommand::Server { port } => {
            run_server(port, connection_config);
        }
        Subcommand::Client { server_address } => {
            run_client(server_address, connection_config);
        }
    }
}

fn run_client(server_address: SocketAddr, connection_config: ConnectionConfig) {
    let authentication = ClientAuthentication::Unsecure {
        protocol_id: 0,
        client_id: 0,
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
        .add_systems(Update, (client_send, client_receive))
        .add_systems(Update, close_on_esc)
        .run();
}

fn run_server(port: u16, connection_config: ConnectionConfig) {
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
        .insert_resource(RenetServer::new(connection_config))
        .insert_resource(NetcodeServerTransport::new(server_config, socket).unwrap())
        .add_systems(
            Update,
            (
                server_broadcast,
                server_receive,
                server_handle_network_events,
            ),
        )
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

fn server_broadcast(mut server: ResMut<RenetServer>) {
    server.broadcast_message(DefaultChannel::ReliableOrdered, "hello from server");
}

fn server_receive(mut server: ResMut<RenetServer>) {
    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, DefaultChannel::ReliableOrdered)
        {
            info!("received message: {:?}", message);
        }
    }
}

fn server_handle_network_events(mut events: EventReader<ServerEvent>) {
    for event in events.read() {
        info!("received event: {:?}", event);
    }
}

fn client_send(mut client: ResMut<RenetClient>) {
    client.send_message(DefaultChannel::ReliableOrdered, "hey from client");
}

fn client_receive(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        info!("received message: {:?}", message);
    }
}
