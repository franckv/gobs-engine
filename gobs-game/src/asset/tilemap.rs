use std::sync::Arc;

use render::color::Color;
use render::model::{Mesh, MeshInstance, MeshInstanceBuilder};
use render::texture::Texture;

use asset::AssetManager;

pub struct Tile {
    instance: MeshInstance
}

impl Tile {
    pub fn instance(&self) -> &MeshInstance {
        &self.instance
    }
}

pub struct TileMap {
    mesh: Arc<Mesh>,
    texture: Arc<Texture>,
    tile_size: [usize; 2],
    size: [usize; 2]
}

impl TileMap {
    pub fn new(asset_manager: &mut AssetManager, texture: Arc<Texture>,
        tile_size: [usize; 2]) -> TileMap {
        let size = {
            let img_size = texture.size();
            [img_size[0] / tile_size[0], img_size[1] / tile_size[1]]
        };

        let mesh = asset_manager.build_quad();

        TileMap {
            mesh: mesh,
            texture: texture,
            tile_size: tile_size,
            size: size
        }
    }

    pub fn build_tile(&self, i: usize, j: usize) -> MeshInstance {
        MeshInstanceBuilder::new(self.mesh.clone())
        .color(Color::white())
        .texture(self.texture.clone())
        .atlas(i, j, self.tile_size)
        .build()
    }

    pub fn size(&self) -> [usize; 2]{
            self.size
    }
}
