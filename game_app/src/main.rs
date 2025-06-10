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
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct AppState {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    renderer: Renderer,
}

impl AppState {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            renderer,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

#[derive(Default)]
struct App {
    state: Option<AppState>,
    world: World,
    input_manager: InputManager,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("My 2D Engine"))
                    .unwrap(),
            );
            let state = pollster::block_on(AppState::new(window));
            self.state = Some(state);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
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
                            ElementState::Released => self.input_manager.action_released(action),
                        }
                    }
                }
                WindowEvent::CloseRequested => {
                    log::info!("The close button was pressed; stopping");
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    state.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    self.world.insert_resource(self.input_manager.clone());
                    let mut schedule = Schedule::default();
                    schedule.add_systems(player_movement_system);
                    schedule.run(&mut self.world);

                    match state.surface.get_current_texture() {
                        Ok(output) => {
                            let view = output
                                .texture
                                .create_view(&wgpu::TextureViewDescriptor::default());
                            state
                                .renderer
                                .render(&mut self.world, &view, &state.device, &state.queue)
                                .unwrap();
                            output.present();
                        }
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("Error acquiring frame: {:?}", e),
                    }
                }
                _ => {}
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

    let mut world = World::new();
    world.insert_resource(Time::default());

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
    };

    event_loop.run_app(&mut app).unwrap();
}
