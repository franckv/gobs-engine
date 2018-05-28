use std::sync::Arc;
use std::vec::Vec;

use render::color::Color;
use render::font::Font;
use render::model::{Mesh, MeshBuilder, MeshManager};
use render::texture::{Texture, TextureLoader};
use render::context::Context;

pub struct AssetManager {
    texture_loader: TextureLoader,
    mesh_manager: MeshManager,
}

impl AssetManager {
    pub fn new(context: Arc<Context>) -> AssetManager {
        AssetManager {
            texture_loader: TextureLoader::new(context.clone()),
            mesh_manager: MeshManager::new(context)
        }
    }

    pub fn load_texture(&self, path: &str) -> Arc<Texture> {
        Arc::new(self.texture_loader.load_texture(path))
    }

    pub fn load_texture_raw(&self, raw: &Vec<u8>, width: usize, height: usize)
    -> Arc<Texture> {
        Arc::new(self.texture_loader.load_texture_raw(raw, width, height))
    }

    pub fn load_font(&self, size: usize, path: &str) -> Font {
        let font = Font::new(&self.texture_loader, size, path);

        font
    }

    pub fn get_color_texture(&self, color: Color) -> Arc<Texture> {
        Arc::new(self.texture_loader.load_color(color))
    }

    pub fn get_mesh_builder(&mut self) -> MeshBuilder {
        self.mesh_manager.get_mesh_builder()
    }

    pub fn build_quad(&mut self) -> Arc<Mesh> {
        let builder = self.mesh_manager.get_mesh_builder();

        let v1 = [-0.5, 0.5, 0.];
        let v2 = [0.5, 0.5, 0.];
        let v3 = [-0.5, -0.5, 0.];
        let v4 = [0.5, -0.5, 0.];

        let n = [0., 0., 1.];

        let t1 = [0., 0.];
        let t2 = [1., 0.];
        let t3 = [0., 1.];
        let t4 = [1., 1.];

        builder
            .add_vertex(v1, n, t1)
            .add_vertex(v3, n, t3)
            .add_vertex(v4, n, t4)
            .add_vertex(v4, n, t4)
            .add_vertex(v2, n, t2)
            .add_vertex(v1, n, t1)
            .build()
    }

    pub fn build_triangle(&mut self) -> Arc<Mesh> {
        let builder = self.mesh_manager.get_mesh_builder();

        let v1 = [-0.5, -0.5, 0.];
        let v2 = [0.5, -0.5, 0.];
        let v3 = [0., 0.5, 0.];

        let n = [0., 0., 1.];

        let t1 = [0., 1.];
        let t2 = [1., 1.];
        let t3 = [0.5, 0.];

        builder
            .add_vertex(v1, n, t1)
            .add_vertex(v2, n, t2)
            .add_vertex(v3, n, t3)
            .build()
    }

    pub fn build_cube(&mut self) -> Arc<Mesh> {
        let builder = self.mesh_manager.get_mesh_builder();

/*
            5 ----- 6
        1 ----- 2   |
        |   |   |   |
        |   7 --|-- 8
        3 ----- 4
*/

        let v1 = [-0.5, 0.5, 0.5];
        let v2 = [0.5, 0.5, 0.5];
        let v3 = [-0.5, -0.5, 0.5];
        let v4 = [0.5, -0.5, 0.5];
        let v5 = [-0.5, 0.5, -0.5];
        let v6 = [0.5, 0.5, -0.5];
        let v7 = [-0.5, -0.5, -0.5];
        let v8 = [0.5, -0.5, -0.5];

        let n1 = [0., 0., 1.];
        let n2 = [0., 0., -1.];
        let n3 = [-1., 0., 0.];
        let n4 = [1., 0., 0.];
        let n5 = [0., 1., 0.];
        let n6 = [0., -1., 0.];

        let t1 = [0., 0.];
        let t2 = [1., 0.];
        let t3 = [0., 1.];
        let t4 = [1., 1.];

        builder
            // F
            .add_vertex(v3, n1, t3)
            .add_vertex(v4, n1, t4)
            .add_vertex(v2, n1, t2)
            .add_vertex(v3, n1, t3)
            .add_vertex(v2, n1, t2)
            .add_vertex(v1, n1, t1)

            // B
            .add_vertex(v8, n2, t3)
            .add_vertex(v7, n2, t4)
            .add_vertex(v5, n2, t2)
            .add_vertex(v8, n2, t3)
            .add_vertex(v5, n2, t2)
            .add_vertex(v6, n2, t1)

            // L
            .add_vertex(v7, n3, t3)
            .add_vertex(v3, n3, t4)
            .add_vertex(v1, n3, t2)
            .add_vertex(v7, n3, t3)
            .add_vertex(v1, n3, t2)
            .add_vertex(v5, n3, t1)

            // R
            .add_vertex(v4, n4, t3)
            .add_vertex(v8, n4, t4)
            .add_vertex(v6, n4, t2)
            .add_vertex(v4, n4, t3)
            .add_vertex(v6, n4, t2)
            .add_vertex(v2, n4, t1)

            // U
            .add_vertex(v1, n5, t3)
            .add_vertex(v2, n5, t4)
            .add_vertex(v6, n5, t2)
            .add_vertex(v1, n5, t3)
            .add_vertex(v6, n5, t2)
            .add_vertex(v5, n5, t1)

            //D
            .add_vertex(v7, n6, t3)
            .add_vertex(v8, n6, t4)
            .add_vertex(v4, n6, t2)
            .add_vertex(v7, n6, t3)
            .add_vertex(v4, n6, t2)
            .add_vertex(v3, n6, t1)

            .build()
    }
}
