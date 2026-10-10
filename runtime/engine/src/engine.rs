use function::window::{WindowConfig, WindowHandler, WindowSystem};
use std::time::Instant;

const WINDOW_TITLE: &str = "CuEngine";
const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const FPS_SMOOTHING: f32 = 1.0 / 100.0;
const MAX_DELTA_TIME: f32 = 0.1;

pub struct Engine {
    last_tick_time: Instant,
    average_duration: f32,
    frame_count: u64,
    fps: u32,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            last_tick_time: Instant::now(),
            average_duration: 0.0,
            frame_count: 0,
            fps: 0,
        }
    }

    pub fn run(mut self) {
        engine_core::log_info!("engine start");

        let config = WindowConfig::new(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
        function::window::run(&mut self, config);

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
    }

    fn on_update(&mut self, window: &WindowSystem) {
        let delta_time = self.calculate_delta_time();
        self.calculate_fps(delta_time);

        window.set_title(&format!("{} - {} FPS", WINDOW_TITLE, self.fps));
        // 迭代 1：在此渲染
    }
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
