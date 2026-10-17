use super::object::{GObject, GObjectId};

pub struct Level {
    objects: Vec<GObject>,
    next_object_id: GObjectId,
}

impl Level {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            next_object_id: 0,
        }
    }

    pub fn create_object(&mut self, name: impl Into<String>) -> &mut GObject {
        let id = self.next_object_id;
        self.next_object_id += 1;
        self.objects.push(GObject::new(id, name));
        self.objects.last_mut().expect("object was just pushed")
    }

    pub fn objects(&self) -> &[GObject] {
        &self.objects
    }

    pub fn tick(&mut self, delta_time: f32) {
        for object in &mut self.objects {
            object.tick(delta_time);
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use engine_core::glam::Vec2;

    use super::*;
    use crate::framework::component::Mover;

    #[test]
    fn level_ticks_objects() {
        let mut level = Level::new();
        level
            .create_object("player")
            .add_component(Box::new(Mover::new(Vec2::new(1.0, 0.0), -10.0, 10.0)));

        level.tick(1.0);

        assert_eq!(level.objects()[0].transform().position.x, 1.0);
    }

    #[test]
    fn object_ids_are_unique() {
        let mut level = Level::new();
        let first = level.create_object("a").id();
        let second = level.create_object("b").id();
        assert_ne!(first, second);
    }
}
