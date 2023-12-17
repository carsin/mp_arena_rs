use bevy::{ecs::system::SystemParam, prelude::*, sprite::Mesh2dHandle};

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub mesh: ColorMesh2dBundle,
}

#[derive(SystemParam)]
pub struct PlayerBundleFactory<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

impl PlayerBundleFactory<'_> {
    pub fn build(&mut self) -> PlayerBundle {
        PlayerBundle {
            player: Player,
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

#[derive(Component, Default)]
pub struct Player;
