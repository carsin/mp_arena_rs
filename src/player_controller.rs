use bevy::{prelude::*, window::PrimaryWindow};

pub const PLAYER_SPEED: f32 = 15.0;

pub struct PlayerControllerPlugin;

impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
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
            FixedUpdate,
            (read_controls, apply_controls.after(read_controls)),
        );
    }
}

#[derive(Component, Default)]
pub struct PlayerController {
    /// Direction player is trying to move. Magnitude shall always be less than
    /// or equal to 1.
    move_direction: Vec2,

    /// Current player motion. May be replaced by a physics plugin later.
    velocity: Vec2,

    /// Angle player is trying to face towards.
    target_angle: f32,
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
