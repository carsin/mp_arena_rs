mod player;

pub use player::{Bundle as PlayerRendererBundle, Factory as PlayerRendererBundleFactory};

use bevy::prelude::*;

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player::update);
    }
}

pub fn interpolate_transform(original: &mut Transform, target: &Transform, factor: f32) {
    original.translation = original.translation.lerp(target.translation, factor);
    original.rotation = original.rotation.slerp(target.rotation, factor);
    original.scale = original.scale.lerp(target.scale, factor);
}
