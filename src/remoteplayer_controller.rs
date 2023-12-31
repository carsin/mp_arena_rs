use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct RemotePlayerControllerPlugin;

impl Plugin for RemotePlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

#[derive(Component, Default, Serialize, Deserialize, Debug, Clone)]
pub struct RemotePlayerController {
    pub server_position: Vec2,
    pub server_angle: f32,
}

fn update(time: Res<Time>, mut players: Query<(&mut RemotePlayerController, &mut Transform)>) {
    for (controller, mut transform) in players.iter_mut() {
        transform.translation = transform.translation.lerp(
            controller.server_position.extend(0.0),
            time.delta_seconds() * 10.0,
        );

        transform.rotation = transform.rotation.slerp(
            Quat::from_rotation_z(controller.server_angle),
            time.delta_seconds() * 10.0,
        );
    }
}
