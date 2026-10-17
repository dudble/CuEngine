use engine_core::glam::Vec2;

use super::transform::Transform2D;

pub trait Component {
    fn type_name(&self) -> &'static str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn tick(&mut self, transform: &mut Transform2D, delta_time: f32);
}

pub struct Mover {
    velocity: Vec2,
    min_x: f32,
    max_x: f32,
}

impl Mover {
    pub fn new(velocity: Vec2, min_x: f32, max_x: f32) -> Self {
        Self {
            velocity,
            min_x,
            max_x,
        }
    }

    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    pub fn min_x(&self) -> f32 {
        self.min_x
    }

    pub fn max_x(&self) -> f32 {
        self.max_x
    }
}

impl Component for Mover {
    fn type_name(&self) -> &'static str {
        "Mover"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick(&mut self, transform: &mut Transform2D, delta_time: f32) {
        transform.translate(self.velocity * delta_time);

        if transform.position.x > self.max_x {
            transform.position.x = self.max_x;
            self.velocity.x = -self.velocity.x.abs();
        } else if transform.position.x < self.min_x {
            transform.position.x = self.min_x;
            self.velocity.x = self.velocity.x.abs();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mover_translates_transform() {
        let mut mover = Mover::new(Vec2::new(2.0, 0.0), -10.0, 10.0);
        let mut transform = Transform2D::default();

        mover.tick(&mut transform, 1.0);

        assert_eq!(transform.position.x, 2.0);
    }

    #[test]
    fn mover_bounces_at_bounds() {
        let mut mover = Mover::new(Vec2::new(10.0, 0.0), -1.0, 1.0);
        let mut transform = Transform2D::default();

        mover.tick(&mut transform, 1.0);

        assert_eq!(transform.position.x, 1.0);
        assert_eq!(mover.velocity.x, -10.0);
    }
}
