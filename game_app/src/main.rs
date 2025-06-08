use engine_core::{Color, math::vec2};
use engine_ecs::{
    components::{MoveSpeed, Player, Renderable, Transform},
    prelude::*,
    systems::player_movement_system,
};
use engine_input::{InputAction, InputManager};
use engine_renderer::Renderer;
use engine_time::Time;
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct App {
    state: Option<AppState>,
    world: World,
    input_manager: InputManager,
    last_update: Option<Instant>,
}

struct AppState {
    window: Arc<Window>,
    renderer: Renderer,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("My 2D Engine"))
                    .unwrap(),
            );

            let renderer = pollster::block_on(Renderer::new(window.clone()));

            window.request_redraw();

            self.state = Some(AppState { window, renderer });

            self.last_update = Some(Instant::now());
            self.world.insert_resource(Time::default());
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            if !state.renderer.input(&event) {
                match event {
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(keycode),
                                state,
                                ..
                            },
                        ..
                    } => {
                        let action = match keycode {
                            KeyCode::KeyW | KeyCode::ArrowUp => Some(InputAction::MoveForward),
                            KeyCode::KeyS | KeyCode::ArrowDown => Some(InputAction::MoveBack),
                            KeyCode::KeyA | KeyCode::ArrowLeft => Some(InputAction::MoveLeft),
                            KeyCode::KeyD | KeyCode::ArrowRight => Some(InputAction::MoveRight),
                            _ => None,
                        };

                        if let Some(action) = action {
                            match state {
                                ElementState::Pressed => self.input_manager.action_pressed(action),
                                ElementState::Released => {
                                    self.input_manager.action_released(action)
                                }
                            }
                        }
                    }
                    WindowEvent::CloseRequested => {
                        log::info!("The close button was pressed; stopping");
                        event_loop.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.renderer.resize(physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        // log::info!("RedrawRequested event received. Preparing to render...");
                        if let Some(last_update) = self.last_update.as_mut() {
                            let now = Instant::now();
                            let delta = now.duration_since(*last_update);
                            *last_update = now;
                            if let Some(mut time) = self.world.get_resource_mut::<Time>() {
                                time.advance_by(delta);
                            }
                        }

                        self.world.insert_resource(self.input_manager.clone());

                        let mut schedule = Schedule::default();
                        schedule.add_systems(player_movement_system);
                        schedule.run(&mut self.world);

                        match state.renderer.render(&mut self.world) {
                            Ok(_) => {}
                            Err(e) => eprintln!("Renderer error: {:?}", e),
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.state.as_ref() {
            state.window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut world = World::new();

    world.spawn((
        Player,
        MoveSpeed(1.0),
        Transform {
            position: vec2(0.0, 0.0),
            scale: vec2(0.2, 0.2),
            rotation: 0.0,
        },
        Renderable {
            color: Color::GREEN,
        },
    ));

    world.spawn((
        Transform {
            position: vec2(0.7, 0.7),
            scale: vec2(0.2, 0.2),
            rotation: 0.0,
        },
        Renderable { color: Color::BLUE },
    ));

    let mut app = App {
        state: None,
        world,
        input_manager: InputManager::default(),
        last_update: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
