use std::sync::Arc;

use engine_core::glam::{Mat4, Vec2};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{
    DynamicState, GraphicsPipeline, PipelineBindPoint, PipelineLayout,
    PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::{RenderPass, Subpass};
use vulkano::shader::spirv::bytes_to_words;
use vulkano::shader::{ShaderModule, ShaderModuleCreateInfo};

use super::context::VulkanContext;
use crate::render::camera::Camera2D;
use crate::render::shader;

#[derive(BufferContents, Vertex)]
#[repr(C)]
struct SpriteVertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
    #[format(R32G32_SFLOAT)]
    uv: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
}

#[derive(BufferContents)]
#[repr(C)]
struct CameraUniform {
    view_projection: [[f32; 4]; 4],
}

pub struct SpriteRenderer {
    pipeline: Arc<GraphicsPipeline>,
    pipeline_layout: Arc<PipelineLayout>,
    descriptor_set: Arc<DescriptorSet>,
    vertex_buffer: Subbuffer<[SpriteVertex]>,
    index_buffer: Subbuffer<[u32]>,
    uniform_buffer: Subbuffer<CameraUniform>,
    camera: Camera2D,
    viewport: Viewport,
}

impl SpriteRenderer {
    pub fn new(context: &VulkanContext, render_pass: &Arc<RenderPass>, extent: (u32, u32)) -> Self {
        let device = &context.device;
        let memory_allocator = context.memory_allocator.clone();

        let vertices = [
            SpriteVertex {
                position: [-0.5, -0.5],
                uv: [0.0, 1.0],
                color: [0.9, 0.4, 0.3, 1.0],
            },
            SpriteVertex {
                position: [0.5, -0.5],
                uv: [1.0, 1.0],
                color: [0.3, 0.7, 0.9, 1.0],
            },
            SpriteVertex {
                position: [0.5, 0.5],
                uv: [1.0, 0.0],
                color: [0.9, 0.8, 0.2, 1.0],
            },
            SpriteVertex {
                position: [-0.5, 0.5],
                uv: [0.0, 0.0],
                color: [0.4, 0.9, 0.5, 1.0],
            },
        ];
        let host_visible = AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        let vertex_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            host_visible.clone(),
            vertices,
        )
        .expect("failed to create vertex buffer");

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::from_iter(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            host_visible.clone(),
            indices,
        )
        .expect("failed to create index buffer");

        let uniform_buffer = Buffer::from_data(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            host_visible,
            CameraUniform {
                view_projection: Mat4::IDENTITY.to_cols_array_2d(),
            },
        )
        .expect("failed to create uniform buffer");

        let vs_words = bytes_to_words(shader::SPRITE_VERT).expect("invalid vertex SPIR-V");
        let fs_words = bytes_to_words(shader::SPRITE_FRAG).expect("invalid fragment SPIR-V");
        let vs_module = unsafe {
            ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(&vs_words))
                .expect("failed to create vertex shader module")
        };
        let fs_module = unsafe {
            ShaderModule::new(device.clone(), ShaderModuleCreateInfo::new(&fs_words))
                .expect("failed to create fragment shader module")
        };
        let vs = vs_module.entry_point("main").expect("missing vertex entry point");
        let fs = fs_module
            .entry_point("main")
            .expect("missing fragment entry point");

        let vertex_input_state = SpriteVertex::per_vertex()
            .definition(&vs)
            .expect("failed to build vertex input state");
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let pipeline_layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .expect("failed to build pipeline layout create info"),
        )
        .expect("failed to create pipeline layout");

        let descriptor_set = DescriptorSet::new(
            context.descriptor_set_allocator.clone(),
            pipeline_layout.set_layouts()[0].clone(),
            [WriteDescriptorSet::buffer(0, uniform_buffer.clone())],
            [],
        )
        .expect("failed to create descriptor set");

        let subpass = Subpass::from(render_pass.clone(), 0).expect("failed to get subpass");
        let pipeline = GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(pipeline_layout.clone())
            },
        )
        .expect("failed to create graphics pipeline");

        let camera = Camera2D::new(Vec2::new(extent.0 as f32, extent.1 as f32));
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [extent.0 as f32, extent.1 as f32],
            depth_range: 0.0..=1.0,
        };

        Self {
            pipeline,
            pipeline_layout,
            descriptor_set,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            camera,
            viewport,
        }
    }

    pub fn update_camera(&mut self, width: u32, height: u32) {
        self.camera.set_viewport(width, height);
        self.viewport.extent = [width as f32, height as f32];

        let mut uniform = self
            .uniform_buffer
            .write()
            .expect("failed to write uniform buffer");
        uniform.view_projection = self.camera.view_projection().to_cols_array_2d();
    }

    pub fn record(&self, builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>) {
        builder
            .set_viewport(0, [self.viewport.clone()].into_iter().collect())
            .expect("failed to set viewport")
            .bind_pipeline_graphics(self.pipeline.clone())
            .expect("failed to bind pipeline")
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .expect("failed to bind vertex buffer")
            .bind_index_buffer(self.index_buffer.clone())
            .expect("failed to bind index buffer")
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline_layout.clone(),
                0,
                self.descriptor_set.clone(),
            )
            .expect("failed to bind descriptor set");

        unsafe {
            builder
                .draw_indexed(self.index_buffer.len() as u32, 1, 0, 0, 0)
                .expect("failed to draw");
        }
    }
}
