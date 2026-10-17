use engine_core::glam::Vec2;

#[derive(Debug, Clone)]
pub struct Transform2D {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

impl Transform2D {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            ..Default::default()
        }
    }

    pub fn translate(&mut self, delta: Vec2) {
        self.position += delta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_transform() {
        let transform = Transform2D::default();
        assert_eq!(transform.position, Vec2::ZERO);
        assert_eq!(transform.rotation, 0.0);
        assert_eq!(transform.scale, Vec2::ONE);
    }

    #[test]
    fn translate_moves_position() {
        let mut transform = Transform2D::new(Vec2::new(1.0, 2.0));
        transform.translate(Vec2::new(0.5, -0.5));
        assert_eq!(transform.position, Vec2::new(1.5, 1.5));
    }
}
