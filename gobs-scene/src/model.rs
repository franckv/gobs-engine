pub struct MeshData {
    pub buffer: u64,
    pub offset: usize,
    pub len: usize,
}

pub struct Model {
    pub meshes: Vec<MeshData>,
}
