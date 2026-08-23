#[cfg(test)]
mod tests {
    use gobs_core::{ConfigWriter as _, GobsConfig, ImageExtent2D, ImageFormat};
    use gobs_render_graph::{GfxContext, SceneDataLayout};
    use gobs_render_hal::{
        AlignMode, ImageUsage, ObjectDataLayout, RenderHalConfig, VertexAttribute,
    };
    use gobs_resource::{ResourceLifetime, ResourceManager};

    use crate::{
        Material, MaterialInstance, MaterialInstanceLoader, MaterialLoader, MaterialProperties,
        Pipeline, PipelineLoader, RenderConfig, Texture, TextureLoader, data::TextureDataProp,
    };

    fn setup() -> (GfxContext, ResourceManager) {
        let mut config = GobsConfig::default();
        config.register::<RenderConfig>();
        config.register::<RenderHalConfig>();

        let mut gfx = GfxContext::new("harness", None, config, true);

        let mut resource_manager = ResourceManager::new(gfx.frames_in_flight());
        let texture_loader = TextureLoader::new(&mut gfx);
        resource_manager.register_resource::<Texture>(texture_loader);
        let pipeline_loader = PipelineLoader::new();
        resource_manager.register_resource::<Pipeline>(pipeline_loader);
        let material_loader = MaterialLoader::new();
        resource_manager.register_resource::<Material>(material_loader);
        let material_instance_loader = MaterialInstanceLoader::new();
        resource_manager.register_resource::<MaterialInstance>(material_instance_loader);

        (gfx, resource_manager)
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn texture_array_allocation() {
        let (mut gfx, _resource_manager) = setup();

        let hal = gfx.hal_mut();

        let extent = ImageExtent2D::new(4, 4);

        let mut images = Vec::new();

        for i in 0..4 {
            let image = hal.create_image(
                &format!("image {i}"),
                ImageFormat::R8g8b8a8Unorm,
                ImageUsage::Texture,
                extent,
            );
            let index = hal.allocate_texture_index();
            hal.update_texture_index(index, image);
            images.push((index, image));
        }

        for (index, image) in images {
            hal.release_texture_index(index);
            hal.destroy_image(image);
        }
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn texture_index_recycling() {
        let (mut gfx, _resource_manager) = setup();

        let hal = gfx.hal_mut();

        let extent = ImageExtent2D::new(4, 4);

        let image1 = hal.create_image(
            "image1",
            ImageFormat::R8g8b8a8Unorm,
            ImageUsage::Texture,
            extent,
        );
        let index1 = hal.allocate_texture_index();
        hal.update_texture_index(index1, image1);

        hal.release_texture_index(index1);
        hal.destroy_image(image1);

        let image2 = hal.create_image(
            "image2",
            ImageFormat::R8g8b8a8Unorm,
            ImageUsage::Texture,
            extent,
        );
        let index2 = hal.allocate_texture_index();

        assert_eq!(index1, index2);

        hal.update_texture_index(index2, image2);

        hal.release_texture_index(index2);
        hal.destroy_image(image2);
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn material_array_size_mismatch() {
        let (mut gfx, mut resource_manager) = setup();

        let build_material = |array_size: u32| {
            MaterialProperties::new(
                "test",
                "test.spv",
                "vertex_main",
                "test.spv",
                "fragment_main",
                VertexAttribute::empty(),
                ObjectDataLayout::new(AlignMode::Std140),
                SceneDataLayout::default(),
                ImageFormat::R8g8b8a8Unorm,
                ImageFormat::D32Sfloat,
            )
            .textures(&[TextureDataProp::Diffuse], true, array_size)
        };

        let good_handle =
            resource_manager.add::<Material>(build_material(256), ResourceLifetime::Static, false);
        let good_pipeline = resource_manager
            .get_data::<Material>(gfx.hal_mut(), &good_handle)
            .expect("material load")
            .data
            .pipeline;
        resource_manager
            .get_data::<Pipeline>(gfx.hal_mut(), &good_pipeline)
            .expect("pipeline build");

        let bad_handle =
            resource_manager.add::<Material>(build_material(4), ResourceLifetime::Static, false);
        let bad_pipeline = resource_manager
            .get_data::<Material>(gfx.hal_mut(), &bad_handle)
            .expect("material load")
            .data
            .pipeline;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            resource_manager
                .get_data::<Pipeline>(gfx.hal_mut(), &bad_pipeline)
                .is_ok()
        }));

        assert!(matches!(result, Ok(false) | Err(_)));
    }
}
