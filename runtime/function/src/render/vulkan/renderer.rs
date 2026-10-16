use std::sync::Arc;

use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer, RenderPassBeginInfo,
    SubpassBeginInfo, SubpassEndInfo,
};
use vulkano::format::ClearValue;
use vulkano::swapchain::{self, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use winit::window::Window;

use super::context::VulkanContext;
use super::sprite::SpriteRenderer;
use super::swapchain::SwapchainContext;

const CLEAR_COLOR: [f32; 4] = [0.10, 0.10, 0.15, 1.0];

pub struct VulkanRenderer {
    context: VulkanContext,
    swapchain: SwapchainContext,
    sprite: SpriteRenderer,
}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>, vsync: bool) -> Self {
        let size = window.inner_size();
        let window_size = (size.width, size.height);

        let context = VulkanContext::new(window);
        let swapchain = SwapchainContext::new(&context, window_size, vsync);
        let sprite = SpriteRenderer::new(&context, &swapchain.render_pass, window_size);

        Self {
            context,
            swapchain,
            sprite,
        }
    }

    pub fn render_frame(&mut self, window_size: (u32, u32)) {
        if window_size.0 == 0 || window_size.1 == 0 {
            return;
        }

        let extent = [window_size.0, window_size.1];
        if extent != self.swapchain.extent() {
            self.swapchain.recreate(&self.context, extent);
        }

        self.sprite.update_camera(window_size.0, window_size.1);

        let (image_index, _suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.swapchain.clone(), None) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!("[render] acquire_next_image failed: {error:?}; recreating swapchain");
                    self.swapchain.recreate(&self.context, extent);
                    return;
                }
            };

        let command_buffer = self.record_scene(image_index as usize);

        let future = acquire_future
            .then_execute(self.context.queue.clone(), command_buffer)
            .expect("failed to submit command buffer")
            .then_swapchain_present(
                self.context.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.swapchain.clone(),
                    image_index,
                ),
            )
            .then_signal_fence_and_flush()
            .expect("failed to flush");
        future.wait(None).expect("failed to wait for frame");
    }

    fn record_scene(&self, image_index: usize) -> Arc<PrimaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.context.command_buffer_allocator.clone(),
            self.context.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("failed to begin command buffer");

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some(ClearValue::Float(CLEAR_COLOR))],
                    ..RenderPassBeginInfo::framebuffer(
                        self.swapchain.framebuffers[image_index].clone(),
                    )
                },
                SubpassBeginInfo::default(),
            )
            .expect("failed to begin render pass");

        self.sprite.record(&mut builder);

        builder
            .end_render_pass(SubpassEndInfo::default())
            .expect("failed to end render pass");

        builder.build().expect("failed to build command buffer")
    }
}
