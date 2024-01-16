use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rendering::interpolate_transform;

pub struct RemotePlayerControllerPlugin;

impl Plugin for RemotePlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_players);
    }
}

#[derive(Component, Default, Serialize, Deserialize, Debug, Clone)]
pub struct RemotePlayerState {
    pub position: Vec2,
    pub angle: f32,
}

#[derive(Component, Default, Serialize, Deserialize, Debug, Clone)]
pub struct RemoteBulletState {
    /// Position the bullet was shot from.
    pub origin: Vec2,

    /// Direction the bullet was shot towards.
    pub angle: f32,

    /// Milliseconds since epoch that the bullet was shot.
    pub spawn_time: u64,

    pub speed: f32,
}

fn update_players(time: Res<Time>, mut players: Query<(&mut RemotePlayerState, &mut Transform)>) {
    for (state, mut transform) in players.iter_mut() {
        let mut new_transform = transform.clone();
        new_transform.translation = state.position.extend(transform.translation.z);
        new_transform.rotation = Quat::from_rotation_z(state.angle);

        interpolate_transform(&mut transform, &new_transform, 1.0);
    }
}

// fn update_bullets(time: Res<Time>, mut bullets: Query<(&mut RemoteBulletState, &mut Transform)>) {
//     for (state, mut transform) in bullets.iter_mut() {
//         let age =
//             SystemTime::now().duration_since(UNIX_EPOCH + Duration::from_millis(state.spawn_time));
//         let distance_travelled = age * state.speed;

//         let mut new_transform = transform.clone();
//     }
// }