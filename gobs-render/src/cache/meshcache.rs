use std::sync::Arc;
use std::collections::HashMap;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::sync::GpuFuture;

use uuid::Uuid;

use RenderVertex;
use context::Context;
use scene::model::Mesh;

pub struct MeshCache {
    context: Arc<Context>,
    cache: HashMap<Uuid, MeshCacheEntry>
}

impl MeshCache {
    pub fn new(context: Arc<Context>) -> Self {
        MeshCache {
            context: context,
            cache: HashMap::new()
        }
    }

    pub fn get(&mut self, mesh: Arc<Mesh>) -> &MeshCacheEntry {
        let id = mesh.id();

        if !self.cache.contains_key(&id) {
            let entry = MeshCacheEntry::new(mesh, self.context.clone());
            self.cache.insert(id, entry);
        }

        self.cache.get(&id).unwrap()
    }
}

pub struct MeshCacheEntry {
    vbuf: Arc<ImmutableBuffer<[RenderVertex]>>,
    size: usize
}

impl MeshCacheEntry {
    pub fn new(mesh: Arc<Mesh>, context: Arc<Context>) -> Self {
        let size = mesh.vlist().len();

        let vlist: Vec<RenderVertex> = mesh.vlist().iter().map(|&v| v.into()).collect();

        let (vbuf, future) = ImmutableBuffer::from_iter(vlist.into_iter(),
            BufferUsage::vertex_buffer(), context.queue()).unwrap();

        future.flush().expect("Error allocating vertex buffer");

        MeshCacheEntry {
            vbuf: vbuf,
            size: size
        }
    }

    pub fn buffer(&self) -> Arc<ImmutableBuffer<[RenderVertex]>> {
        self.vbuf.clone()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
