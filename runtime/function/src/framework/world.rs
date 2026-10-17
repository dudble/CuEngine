use super::level::Level;

pub struct World {
    level: Level,
}

impl World {
    pub fn new() -> Self {
        Self {
            level: Level::new(),
        }
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn level_mut(&mut self) -> &mut Level {
        &mut self.level
    }

    pub fn tick(&mut self, delta_time: f32) {
        self.level.tick(delta_time);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
