use std::mem;
use std::sync::Arc;

use uuid::Uuid;

use gobs_scene as scene;

use scene::model::Model;
use scene::model::Vertex;

use super::context::Context;

use crate::backend::buffer::{Buffer, BufferUsage};
use crate::backend::image::{Image, ImageFormat, ImageLayout, ImageUsage};

pub struct ModelCache<V> {
    pub vertex_buffer: Buffer<V>,
    pub index_buffer: Buffer<u32>,
    pub texture: Image,
    pub texture_id: Uuid
}

impl<V: Copy> ModelCache<V> {
    pub fn new(context: &Arc<Context>, model: &Arc<Model>) -> Arc<ModelCache<Vertex>> {
        let mesh = model.mesh();
        let vertices = mesh.vlist();
        let indices = mesh.ilist().as_ref().unwrap();

        let vertex_buffer = Self::create_buffer(context,
                                                &vertices,
                                                BufferUsage::Vertex);

        let index_buffer = Self::create_buffer(context,
                                               &indices,
                                               BufferUsage::Index);

        let texture = model.texture().unwrap();
        let texture_id = texture.id();

        let texture_data = texture.data();
        let texture_size = texture.size();

        let texture = Self::create_texture(context,
                                           texture_data,
                                           ImageFormat::R8g8b8a8Unorm,
                                           texture_size[0] as u32,
                                           texture_size[1] as u32);

        Arc::new(ModelCache {
            vertex_buffer,
            index_buffer,
            texture,
            texture_id
        })
    }

    pub fn create_texture(context: &Arc<Context>,
                          data: &Vec<u8>, format: ImageFormat,
                          width: u32, height: u32) -> Image {
        let mut buffer = Buffer::new(data.len(),
                                 BufferUsage::Staging,
                                 context.device());

        buffer.copy(data);

        let image = Image::new(context.device(),
                               format,
                               ImageUsage::Texture,
                               width, height);

        let mut command_buffer = context.get_command_buffer();

        command_buffer.begin();
        command_buffer.transition_image_layout(&image, ImageLayout::Undefined,
                                               ImageLayout::Transfer);
        command_buffer.copy_buffer_to_image(&buffer, &image, width, height);
        command_buffer.transition_image_layout(&image, ImageLayout::Transfer,
                                               ImageLayout::Shader);
        command_buffer.end();

        command_buffer.submit_now(context.queue(),
                                  None);

        image
    }

    pub fn create_buffer<T: Copy>(context: &Context,
                                  entries: &Vec<T>,
                                  usage: BufferUsage) -> Buffer<T> {
        let entry_count = entries.len();
        let entry_size = entry_count * mem::size_of::<T>();

        let mut staging_buffer = Buffer::new(entry_count,
                                         BufferUsage::Staging,
                                         context.device());
        let buffer = Buffer::new(entry_count,
                                 usage,
                                 context.device());

        staging_buffer.copy(&entries);

        let mut command_buffer = context.get_command_buffer();

        command_buffer.begin();
        command_buffer.copy_buffer(&staging_buffer, &buffer, entry_size);
        command_buffer.end();

        command_buffer.submit_now(context.queue(), None);

        buffer
    }
}
