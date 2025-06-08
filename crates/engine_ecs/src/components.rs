use bevy_ecs::prelude::*;
use engine_core::{math::Vec2, Color};

#[derive(Component, Debug)]
pub struct Transform {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

#[derive(Component, Debug)]
pub struct Renderable {
    pub color: Color,
}