use bevy_ecs::prelude::*;
use engine_time::Time;

use crate::components::{MoveSpeed, Player, Transform};
use engine_input::{InputAction, InputManager};

pub fn player_movement_system(
    time: Res<Time>,
    input: Res<InputManager>,
    mut query: Query<(&mut Transform, &MoveSpeed), With<Player>>,
) {
    log::info!(
        "player_movement_system running. delta_seconds: {}",
        time.delta_seconds()
    );
    for (mut transform, speed) in query.iter_mut() {
        let mut direction = engine_core::math::vec2(0.0, 0.0);

        if input.is_action_pressed(&InputAction::MoveForward) {
            direction.y += 1.0;
        }
        if input.is_action_pressed(&InputAction::MoveBack) {
            direction.y -= 1.0;
        }
        if input.is_action_pressed(&InputAction::MoveLeft) {
            direction.x -= 1.0;
        }
        if input.is_action_pressed(&InputAction::MoveRight) {
            direction.x += 1.0;
        }

        log::info!("Calculated direction: {:?}", direction);

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
            transform.position += direction * speed.0 * time.delta_seconds();
        }
    }
}
