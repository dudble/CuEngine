use engine_core::glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    pub position: Vec2,
    pub size: Vec2,
    pub color: [f32; 4],
}

impl Quad {
    pub fn new(position: Vec2, size: Vec2, color: [f32; 4]) -> Self {
        Self {
            position,
            size,
            color,
        }
    }
}
