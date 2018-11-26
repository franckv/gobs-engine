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
            context,
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
    ibuf: Option<Arc<ImmutableBuffer<[u32]>>>,
    size: usize
}

impl MeshCacheEntry {
    pub fn new(mesh: Arc<Mesh>, context: Arc<Context>) -> Self {
        let size = mesh.vlist().len();

        let vlist: Vec<RenderVertex> = mesh.vlist().iter().map(|&v| v.into()).collect();

        let vbuf = {
            let (vbuf, v_future) =
                ImmutableBuffer::from_iter(vlist.into_iter(),
                                           BufferUsage::vertex_buffer(),
                                           context.queue()).unwrap();

            v_future.flush().expect("Error allocating vertex buffer");

            vbuf
        };

        let ibuf = match mesh.ilist() {
            Some(ilist) => {
                let (ibuf, i_future) =
                    ImmutableBuffer::from_iter(ilist.iter().cloned(),
                                               BufferUsage::index_buffer(),
                                               context.queue()).unwrap();

                i_future.flush().expect("Error allocating index buffer");

                Some(ibuf)
            },
            None => None
        };

        MeshCacheEntry {
            vbuf,
            ibuf,
            size
        }
    }

    pub fn buffer(&self) -> Arc<ImmutableBuffer<[RenderVertex]>> {
        self.vbuf.clone()
    }

    pub fn index(&self) -> Option<Arc<ImmutableBuffer<[u32]>>> {
        self.ibuf.clone()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}
