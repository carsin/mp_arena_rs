mod camera_controller;
mod client;
mod player;
mod player_controller;
mod remoteplayer;
mod remoteplayer_controller;
mod server;

use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::{log::LogPlugin, prelude::*, utils::HashMap, window::close_on_esc};
use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, ServerAuthentication, ServerConfig},
        ClientId, ConnectionConfig, DefaultChannel, RenetClient, RenetServer, ServerEvent,
    },
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use camera_controller::{CameraController, CameraControllerPlugin};
use clap::Parser;
use client::run_client;
use player::PlayerBundleFactory;
use player_controller::{PlayerController, PlayerControllerPlugin};
use renet::transport::{NetcodeClientTransport, NetcodeServerTransport, NetcodeTransportError};
use serde::{Deserialize, Serialize};
use server::{run_server, make_connection_config};

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
    let connection_config = make_connection_config();

    match cli.subcommand {
        Subcommand::Server { port } => {
            run_server(port, connection_config);
        }
        Subcommand::Client { server_address } => {
            run_client(server_address, connection_config);
        }
    }
}
