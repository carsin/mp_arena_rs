use bevy::prelude::*;
use bevy_ggrs::prelude::*;
use crate::player::Player;
use crate::Config;

// Components that should be saved/loaded need to support snapshotting. 
#[derive(Default, Reflect, Component, Clone, Copy, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

const ACCELERATION: f32 = 1.0; 
const FRICTION: f32 = 0.9; 
const MAX_SPEED: f32 = 5.0; 

pub fn move_player_system(
    mut query: Query<(&mut Transform, &mut Velocity, &Player), With<Rollback>>,
    inputs: Res<PlayerInputs<Config>>,
    time: Res<Time>,
) {
    let dt = time.delta().as_secs_f32();

    for (mut t, mut v, p) in query.iter_mut() {
        let direction = inputs[p.handle].0.direction; // Vec2 representing direction

        // Update velocity based on direction
        v.x = direction.x * ACCELERATION * dt;
        v.y = direction.y * ACCELERATION * dt;

        // Constrain velocity
        **v = v.clamp_length_max(MAX_SPEED);

        // Apply velocity
        t.translation.x += v.x * dt;
        t.translation.y += v.y * dt;
    }
}
