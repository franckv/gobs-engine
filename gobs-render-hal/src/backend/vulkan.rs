mod bindings;
mod buffer;
mod command;
pub(crate) mod display;
mod pipeline;
pub(crate) mod registry;

use std::{any::Any, collections::HashMap, sync::Arc};

use winit::window::Window;

use gobs_core::{ImageExtent2D, ImageFormat, SamplerFilter, logger};
use gobs_vulkan as vk;

use crate::{
    BindingGroupLayout, BindingGroupType, CommandBuffer, CommandQueueType, ImageUsage,
    ObjectDataLayout, RenderBackendError, VertexAttribute,
    backend::vulkan::{
        buffer::BufferView,
        pipeline::{VkComputePipelineBuilder, VkGraphicsPipelineBuilder},
    },
    hal::{BufferType, Handle, RenderHAL},
    pipeline::{ComputePipelineBuilder, GraphicsPipelineBuilder},
};

use bindings::BindingRegistry;
use command::VkCommandBuffer;
use display::Display;
use registry::ResourcesRegistry;

pub trait VulkanHALExt {
    fn get(&self) -> &VulkanHAL;
    fn get_mut(&mut self) -> &mut VulkanHAL;
}

impl VulkanHALExt for dyn RenderHAL + '_ {
    fn get(&self) -> &VulkanHAL {
        self.as_any().downcast_ref::<VulkanHAL>().unwrap()
    }

    fn get_mut(&mut self) -> &mut VulkanHAL {
        self.as_any_mut().downcast_mut::<VulkanHAL>().unwrap()
    }
}

pub struct VulkanHAL {
    registry: ResourcesRegistry,
    bindings: BindingRegistry,
    pub display: Display,
    pub graphics_queue: Arc<vk::Queue>,
    pub transfer_queue: Arc<vk::Queue>,
    pub allocator: Arc<vk::Allocator>,
    pub device: Arc<vk::Device>,
    pub instance: Arc<vk::Instance>,
}

impl RenderHAL for VulkanHAL {
    fn new_frame(&mut self, frame_number: usize) {
        self.bindings.reset(frame_number);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn create_buffer(&mut self, name: &str, size: usize, ty: BufferType) -> Handle {
        tracing::debug!(target: logger::RESOURCES, "Create buffer {}, size={}", name, size);

        let usage = match ty {
            BufferType::Vertex => vk::BufferUsage::Vertex,
            BufferType::Index => vk::BufferUsage::Index,
            BufferType::Staging => vk::BufferUsage::Staging,
            BufferType::StagingDst => vk::BufferUsage::StagingDst,
            BufferType::Uniform => vk::BufferUsage::Uniform,
        };

        let buffer = vk::buffers::Buffer::new(
            name,
            size,
            usage,
            self.device.clone(),
            self.allocator.clone(),
        );

        let buffer_view = BufferView {
            buffer: Arc::new(buffer),
            offset: 0,
            len: size,
        };

        self.registry.buffers.insert(buffer_view)
    }

    fn upload_buffer(&mut self, handle: Handle, data: &[u8], offset: u64) {
        let buffer = self.registry.buffers.get_mut(handle).unwrap();

        buffer.buffer.copy(data, buffer.offset + offset);
    }

    fn resize_buffer(&mut self, handle: Handle, size: usize) {
        let buffer = self.registry.buffers.get_mut(handle).unwrap();

        buffer.buffer = Arc::new(vk::buffers::Buffer::new(
            buffer.buffer.label(),
            size,
            buffer.buffer.usage,
            self.device.clone(),
            self.allocator.clone(),
        ));
        buffer.offset = 0;
        buffer.len = size;
    }

    fn get_buffer_address(&self, handle: Handle) -> u64 {
        let buffer = self.registry.buffers.get(handle).unwrap();

        buffer.buffer.address() + buffer.offset
    }

    fn create_image(
        &mut self,
        name: &str,
        format: ImageFormat,
        usage: ImageUsage,
        extent: ImageExtent2D,
    ) -> Handle {
        let image = vk::images::Image::new(
            name,
            self.device.clone(),
            format,
            usage,
            extent,
            self.allocator.clone(),
        );

        self.registry.images.insert(image)
    }

    fn create_sampler(&mut self, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Handle {
        let sampler = vk::images::Sampler::new(self.device.clone(), mag_filter, min_filter);

        self.registry.samplers.insert(sampler)
    }

    fn invalidate_image(&mut self, image: Handle) {
        let image = self.registry.images.get_mut(image).unwrap();

        image.invalidate();
    }

    fn get_image_extent(&self, image: Handle) -> ImageExtent2D {
        let image = self.registry.images.get(image).unwrap();

        image.extent
    }

    fn create_command_buffer(
        &mut self,
        name: &str,
        ty: CommandQueueType,
    ) -> Box<dyn CommandBuffer> {
        let queue = match ty {
            CommandQueueType::Graphics => self.graphics_queue.clone(),
            CommandQueueType::Transfer => self.transfer_queue.clone(),
            _ => unimplemented!(),
        };

        let command_pool = vk::CommandPool::new(self.device.clone(), &queue.family);

        let command = vk::CommandBuffer::new(self.device.clone(), queue, command_pool, name);

        Box::new(VkCommandBuffer {
            command,
            frame_number: 0,
        })
    }

    fn create_graphics_pipeline(&self, name: &str) -> Box<dyn GraphicsPipelineBuilder> {
        Box::new(VkGraphicsPipelineBuilder::new(name, self.device.clone()))
    }

    fn create_compute_pipeline(&self, name: &str) -> Box<dyn ComputePipelineBuilder> {
        Box::new(VkComputePipelineBuilder::new(name, self.device.clone()))
    }

    fn get_pipeline_descriptor_types(&self, pipeline: Handle) -> Vec<BindingGroupType> {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.descriptor_layout.keys().cloned().collect()
    }

    fn get_pipeline_descriptor_layout(
        &self,
        pipeline: Handle,
        binding_group_type: &BindingGroupType,
    ) -> Option<&BindingGroupLayout> {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.descriptor_layout.get(binding_group_type)
    }

    fn get_pipeline_object_layout(&self, pipeline: Handle) -> &ObjectDataLayout {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        &pipeline.push_layout
    }

    fn get_pipeline_vertex_attributes(&self, pipeline: Handle) -> VertexAttribute {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.vertex_attribute
    }

    fn acquire(&mut self, frame: usize) -> Result<(), RenderBackendError> {
        self.display.acquire(&mut self.registry, frame)
    }

    fn present(&mut self) -> Result<(), RenderBackendError> {
        self.display.present(&self.graphics_queue)
    }

    fn resize(&mut self) {
        self.display.resize(&mut self.registry, self.device.clone());
    }

    fn request_redraw(&mut self) {
        match &self.display.surface {
            None => (),
            Some(surface) => {
                surface.window.request_redraw();
            }
        }
    }

    fn is_minimized(&self) -> bool {
        if let Some(surface) = &self.display.surface {
            surface.is_minimized()
        } else {
            false
        }
    }

    fn get_render_target(&self) -> Option<Handle> {
        self.display.get_render_target()
    }

    fn get_extent(&self) -> ImageExtent2D {
        self.display.get_extent(&self.device)
    }

    fn wait(&mut self) {
        self.device.wait();
    }
}

impl VulkanHAL {
    pub fn new(
        name: &str,
        window: Option<Window>,
        frames_in_flight: usize,
        validation: bool,
    ) -> Self {
        let instance = vk::Instance::new(name, 1, window.as_ref(), validation).unwrap();

        let mut display = Display::new(instance.clone(), window);

        let device = Self::create_device(instance.clone(), &display);

        let graphics_queue = device.clone().graphics_queue();
        let transfer_queue = device.clone().transfer_queue();

        let allocator = vk::Allocator::new(device.clone());

        let mut registry = ResourcesRegistry::default();
        let bindings = BindingRegistry::new(frames_in_flight);

        display.init(&mut registry, device.clone(), frames_in_flight);

        Self {
            instance,
            display,
            device,
            graphics_queue,
            transfer_queue,
            allocator,
            registry,
            bindings,
        }
    }

    fn create_device(instance: Arc<vk::Instance>, display: &Display) -> Arc<vk::Device> {
        let expected_features = vk::Features::default()
            .fill_mode_non_solid()
            .shader_draw_parameters()
            .buffer_device_address()
            .descriptor_indexing()
            .dynamic_rendering()
            .scalar_block_layout()
            .synchronization2();

        tracing::info!(target: logger::INIT, "Requested features: {:?}", expected_features);

        let p_device = instance
            .find_adapter(&expected_features, display.surface.as_deref())
            .unwrap();

        tracing::info!(target: logger::INIT, "Using adapter {}", p_device.name);

        vk::Device::new(instance.clone(), p_device, display.surface.as_deref()).unwrap()
    }
}

impl Drop for VulkanHAL {
    fn drop(&mut self) {
        self.device.wait();
    }
}
