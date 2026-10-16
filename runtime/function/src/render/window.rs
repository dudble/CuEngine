use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

impl WindowConfig {
    pub fn new(title: impl Into<String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }
}

pub trait WindowHandler {
    fn on_resume(&mut self, _window: &mut WindowSystem) {}
    fn on_update(&mut self, window: &WindowSystem);
    fn on_close(&mut self) {}
}

pub struct WindowSystem {
    window: Arc<Window>,
    occluded: bool,
}

impl WindowSystem {
    fn create(event_loop: &ActiveEventLoop, config: &WindowConfig) -> Result<Self, OsError> {
        let attributes = WindowAttributes::default()
            .with_title(config.title.clone())
            .with_inner_size(LogicalSize::new(config.width as f64, config.height as f64));
        let window = Arc::new(event_loop.create_window(attributes)?);

        Ok(Self {
            window,
            occluded: false,
        })
    }

    pub(crate) fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn size(&self) -> (u32, u32) {
        if self.occluded {
            return (0, 0);
        }
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }
}

pub fn run<H: WindowHandler>(handler: &mut H, config: WindowConfig) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    let mut app = WindowApp {
        handler,
        config,
        window: None,
    };
    event_loop.run_app(&mut app).expect("event loop failed");
}

struct WindowApp<'a, H: WindowHandler> {
    handler: &'a mut H,
    config: WindowConfig,
    window: Option<WindowSystem>,
}

impl<H: WindowHandler> ApplicationHandler for WindowApp<'_, H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let mut window =
                WindowSystem::create(event_loop, &self.config).expect("failed to create window");
            self.handler.on_resume(&mut window);
            self.window = Some(window);
        }
        event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.handler.on_close();
                event_loop.exit();
            }
            WindowEvent::Occluded(occluded) => {
                if let Some(window) = self.window.as_mut() {
                    window.occluded = occluded;
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    self.handler.on_update(window);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            return;
        };

        let (width, height) = window.size();
        if width == 0 || height == 0 {
            event_loop.set_control_flow(ControlFlow::Wait);
        } else {
            window.window.request_redraw();
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }
}
