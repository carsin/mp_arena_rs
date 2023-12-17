use bevy::prelude::*;

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, track_target);
    }
}

#[derive(Component, Default)]
pub struct CameraController {
    pub target: Option<Entity>,
}

fn track_target(
    mut cameras: Query<(&CameraController, &mut Transform)>,
    other_transforms: Query<&Transform, Without<CameraController>>,
    time: Res<Time>,
) {
    for (controller, mut transform) in cameras.iter_mut() {
        let Some(target) = controller.target else {
            continue;
        };

        let Ok(target_transform) = other_transforms.get(target) else {
            continue;
        };

        // transform.translation = transform
        //     .translation
        //     .lerp(target_transform.translation, time.delta_seconds() * 5.0);

        transform.translation = target_transform.translation;
    }
}
