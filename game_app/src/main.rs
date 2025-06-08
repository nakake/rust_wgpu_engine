use engine_core::{Color, math::vec2};
use engine_ecs::{
    components::{Renderable, Transform},
    prelude::*,
};
use engine_renderer::{RenderError, Renderer};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    state: Option<AppState>,
    world: World,
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
                    .create_window(Window::default_attributes())
                    .unwrap(),
            );
            let renderer = pollster::block_on(Renderer::new(window.clone()));
            self.state = Some(AppState { window, renderer });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(state) = self.state.as_mut() {
            if !state.renderer.input(&event) {
                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("The close button was pressed; stopping");
                        event_loop.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.renderer.resize(physical_size);
                    }
                    WindowEvent::RedrawRequested => match state.renderer.render(&mut self.world) {
                        Ok(_) => {}
                        Err(RenderError::SurfaceLost) => state.renderer.resize(state.renderer.size),
                        Err(RenderError::OutOfMemory) => event_loop.exit(),
                    },
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

    let mut world = World::new();
    world.spawn((
        Transform {
            position: vec2(-0.5, 0.5),
            scale: vec2(0.3, 0.3),
            rotation: 0.5,
        },
        Renderable { color: Color::RED },
    ));
    world.spawn((
        Transform {
            position: vec2(0.5, -0.2),
            scale: vec2(0.4, 0.2),
            rotation: -0.2,
        },
        Renderable { color: Color::BLUE },
    ));

    let mut app = App { state: None, world };

    event_loop.run_app(&mut app).unwrap();
}
