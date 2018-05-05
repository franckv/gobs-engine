use std::sync::Arc;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;

use render::Renderer;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_uv: [f32; 2],
}

impl_vertex!(Vertex, position, normal, tex_uv);

pub struct MeshManager {
    id: usize,
    queue: Arc<Queue>
}

impl MeshManager {
    pub fn new(renderer: &Renderer) -> MeshManager {
        MeshManager {
            id: 0,
            queue: renderer.queue()
        }
    }

    pub fn get_mesh_builder(&mut self) -> MeshBuilder {
        self.id += 1;
        MeshBuilder::new(self.id, self.queue.clone())
    }
}

pub struct MeshBuilder {
    id: usize,
    vlist: Option<Vec<Vertex>>,
    ilist: Option<Vec<u32>>,
    queue: Arc<Queue>
}

impl MeshBuilder {
    fn new(id: usize, queue: Arc<Queue>) -> Self {
        MeshBuilder {
            id: id,
            vlist: Some(Vec::new()),
            ilist: None,
            queue: queue
        }
    }

    pub fn add_vertex(&mut self, position: [f32; 3], normal: [f32; 3], tex_uv: [f32; 2]) {
        let vertex = Vertex { position: position, normal: normal, tex_uv: tex_uv };

        if let Some(ref mut v) = self.vlist {
            v.push(vertex);
        }
    }

    pub fn build(mut self) -> Arc<Mesh> {
        Mesh::new(self.id, self.vlist.take().unwrap(), self.ilist.take(), self.queue.clone())
    }
}

pub struct Mesh {
    id: usize,
    size: usize,
    vbuf: Arc<ImmutableBuffer<[Vertex]>>,
    ibuf: Option<Arc<ImmutableBuffer<[u32]>>>
}

impl Mesh {
    fn new(id: usize, vlist: Vec<Vertex>, ilist: Option<Vec<u32>>, queue: Arc<Queue>)
    -> Arc<Mesh> {
        let size = vlist.len();
        let (vbuf, future) = ImmutableBuffer::from_iter(vlist.into_iter(),
        BufferUsage::vertex_buffer(), queue.clone()).unwrap();

        let mut mesh = Mesh {
            id: id,
            size: size,
            vbuf: vbuf,
            ibuf: None
        };

        match ilist {
            Some(list) => {
                let (ibuf, ifuture) = ImmutableBuffer::from_iter(list.into_iter(),
                BufferUsage::vertex_buffer(), queue.clone()).unwrap();
                future.join(ifuture).flush().expect("Error allocating vertex buffer");
                mesh.ibuf = Some(ibuf);
            },
            None => future.flush().expect("Error allocating vertex buffer")
        }

        Arc::new(mesh)
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn buffer(&self) -> Arc<ImmutableBuffer<[Vertex]>> {
        self.vbuf.clone()
    }

    pub fn indices(&self) -> Option<Arc<ImmutableBuffer<[u32]>>> {
        self.ibuf.clone()
    }
}
