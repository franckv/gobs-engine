mod bindings;
mod buffer;
mod command;
pub(crate) mod display;
mod instance;
mod material;
mod pipeline;
pub(crate) mod registry;
mod stats;
mod textures;

use std::{any::Any, collections::HashMap, sync::Arc};

use winit::{
    dpi::PhysicalPosition,
    window::{CursorGrabMode, Window},
};

use gobs_core::{
    ConfigReader as _, GobsConfig, ImageExtent2D, ImageFormat, SamplerFilter,
    data::data_buffer::SliceBuffer, logger,
};
use gobs_vulkan::{self as vk, Device};

use crate::{
    BindResource, BindingGroupLayout, BindingGroupType, CommandBuffer, CommandQueueType,
    ImageUsage, ObjectDataLayout, RenderBackendError, RenderHalConfig, VertexAttribute,
    backend::vulkan::{
        bindings::BindingRegistry,
        buffer::BufferView,
        command::VkCommandBuffer,
        display::Display,
        instance::InstanceRegistry,
        material::MaterialRegistry,
        pipeline::{VkComputePipelineBuilder, VkGraphicsPipelineBuilder},
        registry::ResourcesRegistry,
        stats::GpuStats,
        textures::TextureRegistry,
    },
    hal::{BufferType, Handle, RenderHAL},
    pipeline::{ComputePipelineBuilder, GraphicsPipelineBuilder},
};

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
    textures: TextureRegistry,
    materials: MaterialRegistry,
    instances: InstanceRegistry,
    frames_in_flight: usize,
    pub display: Display,
    pub graphics_queue: Arc<vk::Queue>,
    pub transfer_queue: Arc<vk::Queue>,
    pub allocator: Arc<vk::Allocator>,
    pub device: Arc<vk::Device>,
    pub instance: Arc<vk::Instance>,
}

impl RenderHAL for VulkanHAL {
    fn new_frame(&mut self, frame_number: usize) {
        let frame_id = self.frame_id(frame_number);
        self.bindings.reset(frame_id);
        self.instances.reset(frame_id);
    }

    fn frame_id(&self, frame_number: usize) -> usize {
        frame_number % self.frames_in_flight
    }

    fn frames_in_flight(&self) -> usize {
        self.frames_in_flight
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn create_buffer(&mut self, name: &str, size: usize, ty: BufferType) -> Handle {
        Self::create_buffer_internal(
            name,
            size,
            ty,
            self.device.clone(),
            self.allocator.clone(),
            &mut self.registry,
        )
    }

    fn upload_buffer(&mut self, buffer: Handle, data: &[u8], offset: u64) {
        let buffer = self.registry.buffers.get_mut(buffer).unwrap();

        buffer.buffer.copy(data, buffer.offset + offset);
    }

    fn upload_buffer_with(
        &mut self,
        buffer: Handle,
        offset: usize,
        len: usize,
        f: &mut dyn FnMut(&mut dyn RenderHAL, &mut SliceBuffer),
    ) {
        let buffer = self.registry.buffers.get(buffer).unwrap();

        debug_assert!(offset + len <= buffer.len);

        let total_offset = buffer.offset as usize + offset;

        let buffer = buffer.buffer.clone();

        buffer.mapped_slice_mut(|slice| {
            let slice_offset = &mut slice[total_offset..total_offset + len];
            let mut buf = SliceBuffer::new(slice_offset);
            f(self, &mut buf)
        });
    }

    fn get_buffer_address(&self, buffer: Handle) -> u64 {
        let buffer = self.registry.buffers.get(buffer).unwrap();

        buffer.buffer.address() + buffer.offset
    }

    fn destroy_buffer(&mut self, buffer: Handle) {
        let _ = self.registry.buffers.remove(buffer);
    }

    fn allocate_material_index(&mut self) -> usize {
        self.materials.reserve_index()
    }

    fn update_material_data(&mut self, index: usize, data: &[u8]) {
        assert!(data.len() <= self.materials.material_size(index));

        let offset = self
            .get_material_offset(index)
            .unwrap_or_else(|| panic!("Material index {} is not allocated", index))
            as u64;

        let buffer = self.materials.get_buffer();

        self.upload_buffer(buffer, data, offset);
    }

    fn release_material_index(&mut self, index: usize) {
        self.materials.free(index);
    }

    fn get_material_offset(&self, index: usize) -> Option<u32> {
        self.materials.get_offset(index)
    }

    fn get_instance_buffer(&self, frame_id: usize) -> Handle {
        self.instances.get_buffer(frame_id)
    }

    fn allocate_instance(&mut self, frame_id: usize, size: usize) -> usize {
        self.instances.allocate(frame_id, size)
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

    fn invalidate_image(&mut self, image: Handle) {
        let image = self.registry.images.get_mut(image).unwrap();

        image.invalidate();
    }

    fn get_image_extent(&self, image: Handle) -> ImageExtent2D {
        let image = self.registry.images.get(image).unwrap();

        image.extent
    }

    fn destroy_image(&mut self, image: Handle) {
        let _ = self.registry.images.remove(image);
    }

    fn allocate_texture_index(&mut self) -> usize {
        self.textures.reserve_index()
    }

    fn update_texture_index(&mut self, index: usize, image: Handle) {
        let updated = self.textures.register_with_index(image, index);

        if updated {
            self.bindings.update_static_ds(
                self.device.clone(),
                &self.registry,
                self.textures.get_binding(),
                textures::TEXTURES_BINDING,
                index,
            );
        }
    }

    fn release_texture_index(&mut self, index: usize) {
        self.textures.free(index);
    }

    fn max_textures(&self) -> usize {
        self.textures.size()
    }

    fn create_sampler(&mut self, mag_filter: SamplerFilter, min_filter: SamplerFilter) -> Handle {
        let sampler = vk::images::Sampler::new(self.device.clone(), mag_filter, min_filter);

        self.registry.samplers.insert(sampler)
    }

    fn destroy_sampler(&mut self, sampler: Handle) {
        let _ = self.registry.samplers.remove(sampler);
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

        Box::new(VkCommandBuffer::new(self.device.clone(), name, queue))
    }

    fn create_graphics_pipeline(&self, name: &str) -> Box<dyn GraphicsPipelineBuilder> {
        Box::new(VkGraphicsPipelineBuilder::new(name, self.device.clone()))
    }

    fn create_compute_pipeline(&self, name: &str) -> Box<dyn ComputePipelineBuilder> {
        Box::new(VkComputePipelineBuilder::new(name, self.device.clone()))
    }

    fn destroy_pipeline(&mut self, pipeline: Handle) {
        let _ = self.registry.pipelines.remove(pipeline);
    }

    fn get_pipeline_descriptor_types(&self, pipeline: Handle) -> Vec<BindingGroupType> {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.descriptor_layout.keys().cloned().collect()
    }

    fn get_pipeline_descriptor_layout(
        &self,
        pipeline: Handle,
        binding_group_type: &BindingGroupType,
    ) -> Option<Arc<BindingGroupLayout>> {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.descriptor_layout.get(binding_group_type).cloned()
    }

    fn get_pipeline_object_layout(&self, pipeline: Handle) -> Arc<ObjectDataLayout> {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.instance_layout.clone()
    }

    fn get_pipeline_push_layout(&self, pipeline: Handle) -> Arc<ObjectDataLayout> {
        let pipeline = self.registry.pipelines.get(pipeline).unwrap();

        pipeline.push_layout.clone()
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

    fn lock_mouse(&mut self, lock: bool) {
        if let Some(surface) = &self.display.surface {
            if lock {
                surface
                    .window
                    .set_cursor_grab(CursorGrabMode::Locked)
                    .or_else(|_| surface.window.set_cursor_grab(CursorGrabMode::Confined));
                let extent = surface.window.inner_size();
                let center = PhysicalPosition::new(extent.width / 2, extent.height / 2);
                surface.window.set_cursor_position(center);
            } else {
                surface.window.set_cursor_grab(CursorGrabMode::None);
            }
            surface.window.set_cursor_visible(!lock);
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

    fn info(&self) {
        tracing::info!(target: logger::MEMORY, "Stats: buffers={}, images={}, samplers={}, pipelines={}",
            self.registry.buffers.len(), self.registry.images.len(), self.registry.samplers.len(), self.registry.pipelines.len());

        for buffer in self.registry.buffers.values() {
            tracing::info!(target: logger::MEMORY, "{}", buffer.buffer.label());
        }

        for image in self.registry.images.values() {
            tracing::info!(target: logger::MEMORY, "{}", &image.label);
        }

        for pool in &self.bindings.per_frame_pools {
            tracing::info!(target: logger::MEMORY, "Per frame pool: {}", pool.len());
        }
        tracing::info!(target: logger::MEMORY, "Static pool: {}", self.bindings.static_pools.len());

        for cache in &self.bindings.per_frame_ds_cache {
            tracing::info!(target: logger::MEMORY, "DS in per frame cache: {}", cache.len());
        }
        tracing::info!(target: logger::MEMORY, "DS in static cache: {}", self.bindings.static_ds_cache.len());
    }

    fn create_gpu_stats(&mut self) -> Handle {
        let stats = GpuStats::new(self.device.clone());

        self.registry.stats.insert(stats)
    }

    fn get_gpu_stats_ms(&mut self, stats: Handle) -> f32 {
        let stats = self.registry.stats.get(stats).unwrap();

        let mut result = [0; 2];
        stats.query_pool.get_query_pool_results(0, &mut result);

        let delta = result[1] - result[0];

        (delta as f32 * stats.query_pool.period) / 1_000_000.
    }

    fn destroy_gpu_stats(&mut self, stats: Handle) {
        let _ = self.registry.stats.remove(stats);
    }
}

impl VulkanHAL {
    pub fn new(name: &str, window: Option<Window>, config: GobsConfig, validation: bool) -> Self {
        let instance = vk::Instance::new(name, 1, window.as_ref(), validation).unwrap();

        let mut display = Display::new(instance.clone(), window);

        let device = Self::create_device(instance.clone(), &display);

        let graphics_queue = device.clone().graphics_queue();
        let transfer_queue = device.clone().transfer_queue();

        let allocator = vk::Allocator::new(device.clone());

        let frames_in_flight = config.get_int(RenderHalConfig::FramesInFlight) as usize;
        let textures_array_size = config.get_int(RenderHalConfig::TextureArraySize) as usize;
        let material_array_size = config.get_int(RenderHalConfig::MaterialArraySize) as usize;
        let material_data_size = config.get_int(RenderHalConfig::MaterialDataSize) as usize;
        let instance_array_size = config.get_int(RenderHalConfig::InstanceArraySize) as usize;
        let instance_data_size = config.get_int(RenderHalConfig::InstanceDataSize) as usize;

        let mut registry = ResourcesRegistry::default();
        let bindings = BindingRegistry::new(frames_in_flight);

        let sampler = vk::images::Sampler::new(
            device.clone(),
            SamplerFilter::FilterLinear,
            SamplerFilter::FilterLinear,
        );
        let sampler = registry.samplers.insert(sampler);

        let textures = TextureRegistry::new(textures_array_size, sampler);

        let material_total_size = material_data_size * material_array_size;

        let material_buffer = Self::create_buffer_internal(
            "Bindless material",
            material_total_size,
            BufferType::Storage,
            device.clone(),
            allocator.clone(),
            &mut registry,
        );

        let materials =
            MaterialRegistry::new(material_buffer, material_data_size, material_array_size);

        let instance_total_size = instance_data_size * instance_array_size;

        let instance_buffers = (0..frames_in_flight)
            .map(|_| {
                Self::create_buffer_internal(
                    "Instance buffer",
                    instance_total_size,
                    BufferType::Instance,
                    device.clone(),
                    allocator.clone(),
                    &mut registry,
                )
            })
            .collect();

        let instances =
            InstanceRegistry::new(instance_buffers, instance_data_size, instance_array_size);

        display.init(&mut registry, device.clone(), frames_in_flight);

        Self {
            registry,
            bindings,
            textures,
            materials,
            instances,
            frames_in_flight,
            display,
            graphics_queue,
            transfer_queue,
            allocator,
            device,
            instance,
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

    fn create_buffer_internal(
        name: &str,
        size: usize,
        ty: BufferType,
        device: Arc<Device>,
        allocator: Arc<vk::Allocator>,
        registry: &mut ResourcesRegistry,
    ) -> Handle {
        tracing::debug!(target: logger::RESOURCES, "Create buffer {}, size={}", name, size);

        let usage = match ty {
            BufferType::Vertex => vk::BufferUsage::Vertex,
            BufferType::Index => vk::BufferUsage::Index,
            BufferType::Instance => vk::BufferUsage::Instance,
            BufferType::Staging => vk::BufferUsage::Staging,
            BufferType::StagingDst => vk::BufferUsage::StagingDst,
            BufferType::Uniform => vk::BufferUsage::Uniform,
            BufferType::Storage => vk::BufferUsage::Storage,
        };

        let buffer = vk::buffers::Buffer::new(name, size, usage, device.clone(), allocator.clone());

        let buffer_view = BufferView {
            buffer: Arc::new(buffer),
            offset: 0,
            len: size,
        };

        registry.buffers.insert(buffer_view)
    }
}

impl Drop for VulkanHAL {
    fn drop(&mut self) {
        self.device.wait();
    }
}
