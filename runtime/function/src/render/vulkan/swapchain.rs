use std::sync::Arc;

use vulkano::image::Image;
use vulkano::image::view::ImageView;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::single_pass_renderpass;
use vulkano::swapchain::{PresentMode, Swapchain, SwapchainCreateInfo};

use super::context::VulkanContext;

pub struct SwapchainContext {
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub image_views: Vec<Arc<ImageView>>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub render_pass: Arc<RenderPass>,
}

impl SwapchainContext {
    pub fn new(context: &VulkanContext, window_size: (u32, u32), vsync: bool) -> Self {
        let device = &context.device;
        let surface = &context.surface;

        let capabilities = device
            .physical_device()
            .surface_capabilities(surface, Default::default())
            .expect("failed to query surface capabilities");

        let (image_format, image_color_space) = device
            .physical_device()
            .surface_formats(surface, Default::default())
            .expect("failed to query surface formats")[0];

        let available_present_modes = device
            .physical_device()
            .surface_present_modes(surface, Default::default())
            .expect("failed to query present modes");

        let present_mode = if vsync {
            PresentMode::Fifo
        } else {
            available_present_modes
                .iter()
                .copied()
                .find(|mode| *mode == PresentMode::Mailbox)
                .or_else(|| {
                    available_present_modes
                        .iter()
                        .copied()
                        .find(|mode| *mode == PresentMode::Immediate)
                })
                .unwrap_or(PresentMode::Fifo)
        };

        let image_extent = capabilities
            .current_extent
            .unwrap_or([window_size.0, window_size.1]);
        let min_image_count = match capabilities.max_image_count {
            Some(max) => 2u32.max(capabilities.min_image_count).min(max),
            None => 2u32.max(capabilities.min_image_count),
        };

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_color_space,
                image_extent,
                image_usage: vulkano::image::ImageUsage::COLOR_ATTACHMENT,
                present_mode,
                ..Default::default()
            },
        )
        .expect("failed to create swapchain");

        let render_pass = single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: image_format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {},
            }
        )
        .expect("failed to create render pass");

        let mut this = Self {
            swapchain,
            images,
            image_views: Vec::new(),
            framebuffers: Vec::new(),
            render_pass,
        };
        this.rebuild_framebuffers();

        this
    }

    pub fn extent(&self) -> [u32; 2] {
        self.swapchain.image_extent()
    }

    pub fn recreate(&mut self, context: &VulkanContext, extent: [u32; 2]) {
        unsafe { context.device.wait_idle() }.expect("failed to wait for device");

        let (swapchain, images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: extent,
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        self.swapchain = swapchain;
        self.images = images;
        self.rebuild_framebuffers();
    }

    fn rebuild_framebuffers(&mut self) {
        self.image_views.clear();
        self.framebuffers.clear();

        for image in &self.images {
            let view = ImageView::new_default(image.clone()).expect("failed to create image view");
            let framebuffer = Framebuffer::new(
                self.render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view.clone()],
                    ..Default::default()
                },
            )
            .expect("failed to create framebuffer");

            self.image_views.push(view);
            self.framebuffers.push(framebuffer);
        }
    }
}
