use engine_core::glam::{Mat4, Vec2, camera::rh::proj::vulkan};

pub struct Camera2D {
    pub position: Vec2,
    pub zoom: f32,
    viewport: Vec2,
}

impl Camera2D {
    pub fn new(viewport: Vec2) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 100.0,
            viewport,
        }
    }

    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport = Vec2::new(width as f32, height as f32);
    }

    pub fn view_projection(&self) -> Mat4 {
        let half_width = self.viewport.x * 0.5 / self.zoom;
        let half_height = self.viewport.y * 0.5 / self.zoom;

        let projection =
            vulkan::orthographic(-half_width, half_width, -half_height, half_height, -1.0, 1.0);
        let view = Mat4::from_translation(-self.position.extend(0.0));

        projection * view
    }
}
