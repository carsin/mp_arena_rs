use bevy::{prelude::*, window::PrimaryWindow};
use serde::{Deserialize, Serialize};

pub const PLAYER_SPEED: f32 = 15.0;

pub struct PlayerControllerPlugin {
    // If headless, player controllers will not be updated using local inputs.
    // Turn this on for server side.
    pub headless: bool,
}

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, apply_controls);

        if !self.headless {
            app.add_systems(FixedUpdate, read_controls.before(apply_controls));
        }
    }
}

#[derive(Component, Clone, Default, Serialize, Deserialize, Debug)]
pub struct PlayerController {
    /// Direction player is trying to move. Magnitude shall always be less than
    /// or equal to 1.
    pub move_direction: Vec2,

    /// Current player motion. May be replaced by a physics plugin later.
    pub velocity: Vec2,

    /// Angle player is trying to face towards.
    pub target_angle: f32,
}

fn read_controls(
    mut controllers: Query<(&mut PlayerController, &GlobalTransform)>,
    keys: Res<Input<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = windows.get_single() else {
        warn!("Missing window for reading controls");
        return;
    };

    let Ok((camera, camera_transform)) = camera.get_single() else {
        warn!("Camera is missing for reading controls");
        return;
    };

    let hovered_position = window
        .cursor_position()
        .and_then(|cursor_pos| camera.viewport_to_world_2d(camera_transform, cursor_pos));

    for (mut controller, controller_transform) in controllers.iter_mut() {
        controller.move_direction = IVec2::new(
            keys.pressed(KeyCode::D) as i32 - keys.pressed(KeyCode::A) as i32,
            keys.pressed(KeyCode::W) as i32 - keys.pressed(KeyCode::S) as i32,
        )
        .as_vec2()
        .normalize_or_zero();

        if let Some(hovered_position) = hovered_position {
            let diff = hovered_position - controller_transform.translation().xy();
            controller.target_angle = diff.y.atan2(diff.x);
        }
    }
}

fn apply_controls(
    mut players: Query<(&mut PlayerController, &mut Transform)>,
    time: Res<Time<Fixed>>,
) {
    for (mut controller, mut transform) in players.iter_mut() {
        // Update velocity
        controller.velocity = controller.velocity.lerp(
            controller.move_direction * PLAYER_SPEED,
            time.delta_seconds() * 5.0,
        );

        // Update position
        transform.translation += (controller.velocity * time.delta_seconds()).extend(0.0);

        // Update angle
        transform.rotation = transform.rotation.lerp(
            Quat::from_rotation_z(controller.target_angle),
            time.delta_seconds() * 5.0,
        );
    }
}
