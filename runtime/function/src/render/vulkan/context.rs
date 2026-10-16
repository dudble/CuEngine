use std::sync::Arc;

use vulkano::VulkanLibrary;
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{
    Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::swapchain::Surface;
use winit::window::Window;

pub struct VulkanContext {
    pub surface: Arc<Surface>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub memory_allocator: Arc<StandardMemoryAllocator>,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl VulkanContext {
    pub fn new(window: Arc<Window>) -> Self {
        let library = VulkanLibrary::new().expect("no Vulkan library available");
        let required_extensions =
            Surface::required_extensions(&*window).expect("failed to get surface extensions");

        #[cfg(debug_assertions)]
        let (layers, extensions) = (
            vec!["VK_LAYER_KHRONOS_validation".to_string()],
            {
                let mut extensions = required_extensions;
                extensions.ext_debug_utils = true;
                extensions
            },
        );
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

        let surface =
            Surface::from_window(instance.clone(), window).expect("failed to create surface");

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

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator =
            Arc::new(StandardDescriptorSetAllocator::new(device.clone(), Default::default()));

        Self {
            surface,
            device,
            queue,
            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
        }
    }
}
