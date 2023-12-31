use bevy::{ecs::system::SystemParam, prelude::*, sprite::Mesh2dHandle};

#[derive(Bundle, Default)]
pub struct PlayerRendererBundle {
    pub mesh: ColorMesh2dBundle,
}

#[derive(SystemParam)]
pub struct PlayerRendererBundleFactory<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<ColorMaterial>>,
}

impl PlayerRendererBundleFactory<'_> {
    pub fn build(&mut self) -> PlayerRendererBundle {
        PlayerRendererBundle {
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
