use bevy::{ecs::system::SystemParam, prelude::*, sprite::Mesh2dHandle};
use bevy_renet::renet::ClientId;

use crate::{player_controller::PlayerController, remote_state::RemotePlayerState};

use super::interpolate_transform;

#[derive(Component, Deref, DerefMut)]
pub struct Renderer {
    pub player: Entity,
}

#[derive(Bundle)]
pub struct Bundle {
    entity: Renderer,
    pub mesh: ColorMesh2dBundle,
}

#[derive(SystemParam)]
pub struct Factory<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

impl Factory<'_> {
    pub fn build(&mut self, player: Entity) -> Bundle {
        Bundle {
            entity: Renderer { player },
            mesh: ColorMesh2dBundle {
                mesh: Mesh2dHandle(
                    self.meshes.add(
                        shape::Quad {
                            size: Vec2::splat(1.2),
                            ..Default::default()
                        }
                        .into(),
                    ),
                ),
                material: self.materials.add(ColorMaterial {
                    color: Color::WHITE,
                    ..Default::default()
                }),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..Default::default()
            },
        }
    }
}

pub fn update(
    players: Query<&Transform, Without<Renderer>>,
    mut renderers: Query<(&Renderer, &mut Transform)>,
) {
    for (player_entity, mut renderer_transform) in renderers.iter_mut() {
        let Ok(player_transform) = players.get(**player_entity) else {
            warn!(
                "Player renderer references a player entity ({:?}) that does not exist",
                **player_entity
            );
            continue;
        };

        interpolate_transform(&mut renderer_transform, player_transform, 1.0);
    }
}
