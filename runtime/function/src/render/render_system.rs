use crate::render::render_scene::RenderScene;
use crate::render::vulkan::VulkanRenderer;
use crate::render::window::WindowSystem;

pub struct RenderSystem {
    renderer: VulkanRenderer,
}

impl RenderSystem {
    pub fn new(window: &WindowSystem, vsync: bool) -> Self {
        Self {
            renderer: VulkanRenderer::new(window.window(), vsync),
        }
    }

    pub fn render_frame(&mut self, window_size: (u32, u32), scene: &RenderScene) {
        self.renderer.render_frame(window_size, scene);
    }
}
