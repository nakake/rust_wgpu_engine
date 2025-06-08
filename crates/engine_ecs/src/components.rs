use bevy_ecs::prelude::*;
use engine_core::{Color, math::Vec2};

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

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct MoveSpeed(pub f32);
