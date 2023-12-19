use bevy::prelude::*;
use bevy_ggrs::prelude::*;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable, Debug, Default)]
pub struct PlayerInput {
    direction: Vec2,
}