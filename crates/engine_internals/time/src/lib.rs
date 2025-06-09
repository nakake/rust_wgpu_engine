use std::time::Duration;

use bevy_ecs::resource::Resource;

#[derive(Resource)]
pub struct Time {
    delta: Duration,
    delta_seconds: f32,
}

impl Time {
    pub fn advance_by(&mut self, delta: Duration) {
        self.delta = delta;
        self.delta_seconds = delta.as_secs_f32();
    }

    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }
}

impl Default for Time {
    fn default() -> Self {
        Self {
            delta: Duration::from_secs(0),
            delta_seconds: 0.0,
        }
    }
}
