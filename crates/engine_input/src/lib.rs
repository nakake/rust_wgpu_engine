use bevy_ecs::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    MoveForward,
    MoveBack,
    MoveLeft,
    MoveRight,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct InputManager {
    pressed_actions: HashSet<InputAction>,
}

impl InputManager {
    pub fn action_pressed(&mut self, key: InputAction) {
        self.pressed_actions.insert(key);
    }

    pub fn action_released(&mut self, key: InputAction) {
        self.pressed_actions.remove(&key);
    }

    pub fn is_action_pressed(&self, key: &InputAction) -> bool {
        self.pressed_actions.contains(key)
    }
}
