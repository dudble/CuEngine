use super::component::Component;
use super::transform::Transform2D;

pub type GObjectId = u64;

pub struct GObject {
    id: GObjectId,
    name: String,
    transform: Transform2D,
    components: Vec<Box<dyn Component>>,
}

impl GObject {
    pub fn new(id: GObjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            transform: Transform2D::default(),
            components: Vec::new(),
        }
    }

    pub fn id(&self) -> GObjectId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    pub fn add_component(&mut self, component: Box<dyn Component>) -> &mut Self {
        self.components.push(component);
        self
    }

    pub fn components(&self) -> &[Box<dyn Component>] {
        &self.components
    }

    pub fn tick(&mut self, delta_time: f32) {
        for component in &mut self.components {
            component.tick(&mut self.transform, delta_time);
        }
    }
}
