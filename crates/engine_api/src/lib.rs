use std::time::Instant;

use engine_ecs::{prelude::*, systems::player_movement_system};
use engine_input::InputManager;
use engine_time::Time;

pub struct Engine {
    pub world: World,
    schedule: Schedule,
    last_update: Instant,
}

impl Engine {
    pub fn new() -> Self {
        log::info!("Headless engine initialization started.");

        let world = World::new();

        let mut schedule = Schedule::default();
        schedule.add_systems(player_movement_system);

        log::info!("Headless engine initialization finished.");
        Self {
            world,
            schedule,
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self, input_manager: &InputManager) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;
        if let Some(mut time) = self.world.get_resource_mut::<Time>() {
            time.advance_by(delta);
        }

        // 2. 最新の入力状態をリソースとして提供
        self.world.insert_resource(input_manager.clone());

        // 3. ECSシステムを実行
        self.schedule.run(&mut self.world);
    }
}
