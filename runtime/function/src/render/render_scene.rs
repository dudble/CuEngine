use super::render_data::Quad;

#[derive(Debug, Default)]
pub struct RenderScene {
    quads: Vec<Quad>,
}

impl RenderScene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.quads.clear();
    }

    pub fn push_quad(&mut self, quad: Quad) {
        self.quads.push(quad);
    }

    pub fn quads(&self) -> &[Quad] {
        &self.quads
    }
}
