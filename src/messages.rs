use bevy::prelude::*;
use bevy::utils::HashMap;
use renet::ClientId;
use serde::{Deserialize, Serialize};

use crate::{
    player_controller::PlayerController,
    remote_state::{RemoteBulletState, RemotePlayerState},
};

/// This ID is assigned by the server and is included in entity synchronization
/// messages as a persistent handle. It can be attached to entities or mapped to
/// entities in a resource.
#[derive(Deref, DerefMut, Component, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub struct NetworkId(pub u32);

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    PlayerConnected { client_id: ClientId },
    PlayerDisconnected { client_id: ClientId },
    Players(HashMap<ClientId, RemotePlayerState>),
    Bullets(HashMap<NetworkId, RemoteBulletState>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Controller(PlayerController),
}
