#[cfg(test)]
mod tests {
    use gobs_core::{ConfigWriter as _, GobsConfig, ImageExtent2D, ImageFormat};
    use gobs_render_hal::{GfxContext, ImageUsage, RenderHalConfig, create_hal};
    use gobs_resource::ResourceManager;

    use crate::{
        Material, MaterialInstance, MaterialInstanceLoader, MaterialLoader, Pipeline,
        PipelineLoader, RenderConfig, Texture, TextureLoader,
    };

    fn setup() -> (Box<GfxContext<'static>>, ResourceManager) {
        let mut config = GobsConfig::default();
        config.register::<RenderConfig>();
        config.register::<RenderHalConfig>();

        let mut ctx = create_hal("harness", None, config, true);

        let mut resource_manager = ResourceManager::new(ctx.frames_in_flight());
        let texture_loader = TextureLoader::new(ctx.as_mut());
        resource_manager.register_resource::<Texture>(texture_loader);
        let pipeline_loader = PipelineLoader::new();
        resource_manager.register_resource::<Pipeline>(pipeline_loader);
        let material_loader = MaterialLoader::new();
        resource_manager.register_resource::<Material>(material_loader);
        let material_instance_loader = MaterialInstanceLoader::new();
        resource_manager.register_resource::<MaterialInstance>(material_instance_loader);

        (ctx, resource_manager)
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn texture_array_allocation() {
        let (mut ctx, _resource_manager) = setup();

        let extent = ImageExtent2D::new(4, 4);

        let mut images = Vec::new();

        for i in 0..4 {
            let image = ctx.create_image(
                &format!("image {i}"),
                ImageFormat::R8g8b8a8Unorm,
                ImageUsage::Texture,
                extent,
            );
            let index = ctx.allocate_texture_index();
            ctx.update_texture_index(index, image);
            images.push((index, image));
        }

        for (index, image) in images {
            ctx.release_texture_index(index);
            ctx.destroy_image(image);
        }
    }

    #[test]
    #[cfg_attr(feature = "ci", ignore)]
    fn texture_index_recycling() {
        let (mut ctx, _resource_manager) = setup();

        let extent = ImageExtent2D::new(4, 4);

        let image1 = ctx.create_image(
            "image1",
            ImageFormat::R8g8b8a8Unorm,
            ImageUsage::Texture,
            extent,
        );
        let index1 = ctx.allocate_texture_index();
        ctx.update_texture_index(index1, image1);

        ctx.release_texture_index(index1);
        ctx.destroy_image(image1);

        let image2 = ctx.create_image(
            "image2",
            ImageFormat::R8g8b8a8Unorm,
            ImageUsage::Texture,
            extent,
        );
        let index2 = ctx.allocate_texture_index();

        assert_eq!(index1, index2);

        ctx.update_texture_index(index2, image2);

        ctx.release_texture_index(index2);
        ctx.destroy_image(image2);
    }
}
