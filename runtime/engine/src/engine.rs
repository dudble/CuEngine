use engine_core::glam::Vec2;
use function::framework::component::Mover;
use function::framework::scene::world_from_level;
use function::framework::world::World;
use function::render::render_system::RenderSystem;
use function::render::window::{WindowConfig, WindowHandler, WindowSystem, run as window_run};
use resource::config::ConfigManager;
use resource::scene::LevelRes;
use std::time::Instant;

use crate::render_bridge;

const DEFAULT_WINDOW_TITLE: &str = "CuEngine";
const FPS_SMOOTHING: f32 = 1.0 / 100.0;
const MAX_DELTA_TIME: f32 = 0.1;

pub struct Engine {
    world: World,
    render: Option<RenderSystem>,
    window_title: String,
    vsync: bool,
    last_tick_time: Instant,
    average_duration: f32,
    frame_count: u64,
    fps: u32,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            render: None,
            window_title: DEFAULT_WINDOW_TITLE.to_string(),
            vsync: true,
            last_tick_time: Instant::now(),
            average_duration: 0.0,
            frame_count: 0,
            fps: 0,
        }
    }

    pub fn run(mut self) {
        let config_manager = ConfigManager::load();
        let config = config_manager.config().clone();

        engine_core::log::set_level(config.log_level);
        self.window_title = config.window.title.clone();
        self.vsync = config.vsync;

        self.world = match resource::scene::load::<LevelRes>() {
            Some(level) => world_from_level(&level),
            None => demo_world(),
        };

        engine_core::log_info!("engine start");

        let window_config = WindowConfig::new(
            config.window.title.clone(),
            config.window.width,
            config.window.height,
        );
        window_run(&mut self, window_config);

        engine_core::log_info!("engine shutdown");
    }

    fn calculate_delta_time(&mut self) -> f32 {
        let now = Instant::now();
        let delta_time = (now - self.last_tick_time).as_secs_f32();
        self.last_tick_time = now;
        delta_time.min(MAX_DELTA_TIME)
    }

    fn calculate_fps(&mut self, delta_time: f32) {
        self.frame_count += 1;
        if self.frame_count == 1 {
            self.average_duration = delta_time;
        } else {
            self.average_duration =
                self.average_duration * (1.0 - FPS_SMOOTHING) + delta_time * FPS_SMOOTHING;
        }

        self.fps = if self.average_duration > 0.0 {
            (1.0 / self.average_duration) as u32
        } else {
            0
        };
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}



impl WindowHandler for Engine {
    fn on_resume(&mut self, window: &mut WindowSystem) {
        let (width, height) = window.size();
        engine_core::log_info!("window created {}x{}", width, height);

        self.render = Some(RenderSystem::new(window, self.vsync));
        engine_core::log_info!("render system initialized");
    }

    fn on_update(&mut self, window: &WindowSystem) {
        let delta_time = self.calculate_delta_time();
        self.calculate_fps(delta_time);

        self.world.tick(delta_time);

        let scene = render_bridge::extract(&self.world);
        if let Some(render) = &mut self.render {
            render.render_frame(window.size(), &scene);
        }

        window.set_title(&format!("{} - {} FPS", self.window_title, self.fps));
    }

    fn on_close(&mut self) {
        self.render = None;
    }
}

fn demo_world() -> World {
    let mut world = World::new();
    world
        .level_mut()
        .create_object("player")
        .add_component(Box::new(Mover::new(Vec2::new(3.0, 0.0), -4.0, 4.0)));
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_fps_is_zero() {
        assert_eq!(Engine::new().fps, 0);
    }

    #[test]
    fn fps_stabilizes() {
        let mut engine = Engine::new();
        for _ in 0..1000 {
            engine.calculate_fps(1.0 / 60.0);
        }
        assert!((59..=61).contains(&engine.fps), "fps={}", engine.fps);
    }
}
