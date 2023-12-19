use bevy::prelude::*;
use bevy_ggrs::{prelude::*, LocalInputs, LocalPlayers, PlayerInputs, GgrsConfig};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;

use super::Config;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Debug, Default)]
pub struct PlayerInput {
    direction: Vec2,
}

pub fn read_local_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    local_players: Res<LocalPlayers>,
) {
    let mut local_inputs = HashMap::new();

    for handle in &local_players.0 {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::W) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction.x += 1.0;
        }

        local_inputs.insert(*handle, PlayerInput { direction });
    }

    commands.insert_resource(LocalInputs::<Config>(local_inputs));
}
