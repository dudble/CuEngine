use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo, SubpassBeginInfo,
    SubpassEndInfo,
};
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::format::ClearValue;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::single_pass_renderpass;
use vulkano::swapchain::{
    self, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
};
use vulkano::sync::GpuFuture;
use winit::window::Window;

const CLEAR_COLOR: [f32; 4] = [0.10, 0.10, 0.15, 1.0];

pub struct VulkanRenderer {
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    image_views: Vec<Arc<ImageView>>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl VulkanRenderer {
    pub fn new(window: Arc<Window>) -> Self {
        let library = VulkanLibrary::new().expect("no Vulkan library available");
        let required_extensions =
            Surface::required_extensions(&*window).expect("failed to get surface extensions");

        #[cfg(debug_assertions)]
        let (layers, extensions) = (vec!["VK_LAYER_KHRONOS_validation".to_string()], {
            let mut extensions = required_extensions;
            extensions.ext_debug_utils = true;
            extensions
        });
        #[cfg(not(debug_assertions))]
        let (layers, extensions) = (Vec::new(), required_extensions);

        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_layers: layers,
                enabled_extensions: extensions,
                ..Default::default()
            },
        )
        .expect("failed to create Vulkan instance");

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("failed to create surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let physical_device = instance
            .enumerate_physical_devices()
            .expect("failed to enumerate physical devices")
            .find(|physical_device| {
                physical_device.supported_extensions().khr_swapchain
                    && physical_device
                        .queue_family_properties()
                        .iter()
                        .enumerate()
                        .any(|(index, family)| {
                            family.queue_flags.contains(QueueFlags::GRAPHICS)
                                && physical_device
                                    .surface_support(index as u32, &surface)
                                    .unwrap_or(false)
                        })
            })
            .expect("no suitable physical device");

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .find(|(index, family)| {
                family.queue_flags.contains(QueueFlags::GRAPHICS)
                    && physical_device
                        .surface_support(*index as u32, &surface)
                        .unwrap_or(false)
            })
            .map(|(index, _)| index as u32)
            .expect("no suitable queue family");

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .expect("failed to create logical device");

        let queue = queues.next().expect("failed to get queue");

        let capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .expect("failed to query surface capabilities");

        let (image_format, image_color_space) = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .expect("failed to query surface formats")[0];

        let present_mode = device
            .physical_device()
            .surface_present_modes(&surface, Default::default())
            .expect("failed to query present modes")
            .into_iter()
            .find(|mode| *mode == PresentMode::Fifo)
            .unwrap_or(PresentMode::Fifo);

        let size = window.inner_size();
        let image_extent = capabilities
            .current_extent
            .unwrap_or([size.width, size.height]);
        let min_image_count = match capabilities.max_image_count {
            Some(max) => 2u32.max(capabilities.min_image_count).min(max),
            None => 2u32.max(capabilities.min_image_count),
        };

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count,
                image_format,
                image_color_space,
                image_extent,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
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

        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let mut renderer = Self {
            device,
            queue,
            swapchain,
            images,
            image_views: Vec::new(),
            framebuffers: Vec::new(),
            render_pass,
            command_buffer_allocator,
        };
        renderer.recreate_framebuffers();

        renderer
    }

    pub fn render_frame(&mut self, window_size: (u32, u32)) {
        if window_size.0 == 0 || window_size.1 == 0 {
            return;
        }

        let extent = [window_size.0, window_size.1];
        if extent != self.swapchain.image_extent() {
            self.recreate_swapchain(extent);
        }

        let (image_index, _suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(result) => result,
                Err(error) => {
                    eprintln!(
                        "[render] acquire_next_image failed: {error:?}; recreating swapchain"
                    );
                    self.recreate_swapchain(extent);
                    return;
                }
            };

        let command_buffer = self.record_clear(image_index as usize);

        let future = acquire_future
            .then_execute(self.queue.clone(), command_buffer)
            .expect("failed to submit command buffer")
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), image_index),
            )
            .then_signal_fence_and_flush()
            .expect("failed to flush");
        future.wait(None).expect("failed to wait for frame");
    }

    fn record_clear(
        &self,
        image_index: usize,
    ) -> Arc<vulkano::command_buffer::PrimaryAutoCommandBuffer> {
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("failed to begin command buffer");

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some(ClearValue::Float(CLEAR_COLOR))],
                    ..RenderPassBeginInfo::framebuffer(self.framebuffers[image_index].clone())
                },
                SubpassBeginInfo::default(),
            )
            .expect("failed to begin render pass")
            .end_render_pass(SubpassEndInfo::default())
            .expect("failed to end render pass");

        builder.build().expect("failed to build command buffer")
    }
    
    fn recreate_swapchain(&mut self, extent: [u32; 2]) {
        unsafe { self.device.wait_idle() }.expect("failed to wait for device");

        let (swapchain, images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: extent,
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        self.swapchain = swapchain;
        self.images = images;
        self.recreate_framebuffers();
    }

    fn recreate_framebuffers(&mut self) {
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
