mod camera_controller;
mod channels;
mod client;
mod messages;
mod player;
mod player_controller;
mod remote_state;
mod rendering;
mod server;

use std::net::SocketAddr;

use bevy::prelude::*;

use clap::Parser;
use client::run_client;
use server::{make_connection_config, run_server};

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
