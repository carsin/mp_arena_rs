use bevy::prelude::*;
use bevy::utils::HashMap;
use renet::ClientId;
use serde::{Deserialize, Serialize};

use crate::{player_controller::PlayerController, remoteplayer_controller::RemotePlayerController};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    PlayerConnected { client_id: ClientId },
    PlayerDisconnected { client_id: ClientId },
    Players(HashMap<ClientId, RemotePlayerController>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Controller(PlayerController),
}
