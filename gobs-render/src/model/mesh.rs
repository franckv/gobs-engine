use std::sync::Arc;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::device::Queue;
use vulkano::sync::GpuFuture;

use context::Context;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_uv: [f32; 2],
}

impl_vertex!(Vertex, position, normal, tex_uv);

#[derive(Copy, Clone)]
pub enum PrimitiveType {
    Triangle,
    Line
}

pub struct MeshManager {
    id: usize,
    context: Arc<Context>
}

impl MeshManager {
    pub fn new(context: Arc<Context>) -> MeshManager {
        MeshManager {
            id: 0,
            context: context
        }
    }

    pub fn get_mesh_builder(&mut self) -> MeshBuilder {
        self.id += 1;
        MeshBuilder::new(self.id, self.context.queue())
    }
}

pub struct MeshBuilder {
    id: usize,
    primitive_type: PrimitiveType,
    vlist: Option<Vec<Vertex>>,
    ilist: Option<Vec<u32>>,
    queue: Arc<Queue>
}

impl MeshBuilder {
    fn new(id: usize, queue: Arc<Queue>) -> Self {
        MeshBuilder {
            id: id,
            primitive_type: PrimitiveType::Triangle,
            vlist: Some(Vec::new()),
            ilist: None,
            queue: queue
        }
    }

    pub fn add_vertex(mut self, position: [f32; 3], normal: [f32; 3], tex_uv: [f32; 2]) -> Self {
        let vertex = Vertex { position: position, normal: normal, tex_uv: tex_uv };

        if let Some(ref mut v) = self.vlist {
            v.push(vertex);
        }

        self
    }

    pub fn line(mut self) -> Self {
        self.primitive_type = PrimitiveType::Line;

        self
    }

    pub fn build(mut self) -> Arc<Mesh> {
        Mesh::new(self.id, self.primitive_type, self.vlist.take().unwrap(), self.ilist.take(), self.queue.clone())
    }
}

pub struct Mesh {
    id: usize,
    size: usize,
    primitive_type: PrimitiveType,
    vbuf: Arc<ImmutableBuffer<[Vertex]>>,
    ibuf: Option<Arc<ImmutableBuffer<[u32]>>>
}

impl Mesh {
    fn new(id: usize, primitive_type: PrimitiveType, vlist: Vec<Vertex>, ilist: Option<Vec<u32>>, queue: Arc<Queue>)
    -> Arc<Mesh> {
        let size = vlist.len();
        let (vbuf, future) = ImmutableBuffer::from_iter(vlist.into_iter(),
        BufferUsage::vertex_buffer(), queue.clone()).unwrap();

        let mut mesh = Mesh {
            id: id,
            size: size,
            primitive_type: primitive_type,
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

    pub fn primitive_type(&self) -> PrimitiveType {
            self.primitive_type
    }

    pub fn buffer(&self) -> Arc<ImmutableBuffer<[Vertex]>> {
        self.vbuf.clone()
    }

    pub fn indices(&self) -> Option<Arc<ImmutableBuffer<[u32]>>> {
        self.ibuf.clone()
    }
}
