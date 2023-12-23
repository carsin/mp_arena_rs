use bevy::{prelude::*, window::PrimaryWindow};

pub const PLAYER_SPEED: f32 = 15.0;

pub struct RemotePlayerControllerPlugin;

impl Plugin for RemotePlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        // TODO: since rollback networking will be used, movements need to
        // be in fixed update. This will appear a little choppy for high
        // refresh displays so should figure out how to interpolate rendered
        // objects.
        //
        // Idea: instead of attaching meshes directly to an entity, add a
        // separate XXXXRenderer entity that has an internal entity
        // reference field tracking the target object that will be rendered.
        // And just interpolate the renderer's transform to the tracked
        // object's transform.

        // FixedUpdate,
        // (read_controls, apply_controls.after(read_controls)),
        // );
    }
}

#[derive(Component, Default)]
pub struct RemotePlayerController {
    /// Current player motion. May be replaced by a physics plugin later.
    velocity: Vec2,
    /// Angle player is trying to face towards.
    target_angle: f32,
}
