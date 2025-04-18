use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use renderdoc::{RenderDoc, V141};

use gobs::{
    core::ImageFormat,
    game::input::{Input, Key},
    render::{
        BlendMode, Context, FrameGraph, Material, MaterialProperty, PassType, RenderError,
        Renderable, RenderableLifetime,
    },
    resource::{entity::camera::Camera, geometry::VertexFlag},
    scene::scene::Scene,
    ui::UIRenderer,
};

use crate::{CameraController, ui::Ui};

pub struct SampleApp {
    pub process_updates: bool,
    pub draw_ui: bool,
    pub draw_bounds: bool,
    pub draw_wire: bool,
    pub ui: Ui,
}

impl SampleApp {
    pub fn new() -> Self {
        tracing::info!("Create");

        Self {
            process_updates: false,
            draw_ui: false,
            draw_bounds: false,
            draw_wire: false,
            ui: Ui::new(),
        }
    }

    pub fn ortho_camera(ctx: &Context) -> Camera {
        let extent = ctx.extent();

        Camera::ortho(extent.width as f32, extent.height as f32, 0.1, 100., 0., 0.)
    }

    pub fn perspective_camera(ctx: &Context) -> Camera {
        let extent = ctx.extent();

        Camera::perspective(
            extent.width as f32 / extent.height as f32,
            60_f32.to_radians(),
            0.1,
            100.,
            0.,
            0.,
        )
    }

    pub fn controller() -> CameraController {
        CameraController::new(3., 0.4)
    }

    pub fn color_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder(ctx, "color.vert.spv", "color.frag.spv")
            .vertex_flags(vertex_flags)
            .build(graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn color_material_transparent(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder(ctx, "color.vert.spv", "color.frag.spv")
            .vertex_flags(vertex_flags)
            .blend_mode(BlendMode::Alpha)
            .build(graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn texture_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder(ctx, "mesh.vert.spv", "mesh.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .build(graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn texture_material_transparent(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder(ctx, "mesh.vert.spv", "mesh.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .blend_mode(BlendMode::Alpha)
            .build(graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn normal_mapping_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION
            | VertexFlag::TEXTURE
            | VertexFlag::NORMAL
            | VertexFlag::TANGENT
            | VertexFlag::BITANGENT;

        Material::builder(ctx, "mesh.vert.spv", "mesh_n.frag.spv")
            .vertex_flags(vertex_flags)
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .build(graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn depth_material(&self, ctx: &Context, graph: &FrameGraph) -> Arc<Material> {
        let vertex_flags = VertexFlag::POSITION | VertexFlag::COLOR;

        Material::builder(ctx, "color.vert.spv", "depth.frag.spv")
            .vertex_flags(vertex_flags)
            .build(graph.pass_by_type(PassType::Forward).unwrap())
    }

    pub fn update_ui(
        &mut self,
        ctx: &Context,
        graph: &FrameGraph,
        scene: &Scene,
        ui: &mut UIRenderer,
        delta: f32,
    ) {
        if self.draw_ui {
            let (camera_transform, camera) = scene.camera();
            ui.update(
                ctx,
                graph.pass_by_type(PassType::Ui).unwrap(),
                delta,
                |ectx| {
                    self.ui
                        .draw(ctx, ectx, graph, scene, camera, &camera_transform);
                },
            );
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut Context,
        graph: &mut FrameGraph,
        scene: &mut Scene,
        ui: &mut UIRenderer,
    ) -> Result<(), RenderError> {
        tracing::trace!("Render frame {}", ctx.frame_number);

        graph.begin(ctx)?;

        graph.prepare(ctx, &mut |pass, batch| match pass.ty() {
            PassType::Depth | PassType::Forward => {
                scene.draw(ctx, pass, batch, None, RenderableLifetime::Static);
            }
            PassType::Wire => {
                if self.draw_wire {
                    scene.draw(ctx, pass, batch, None, RenderableLifetime::Static);
                }
            }
            PassType::Bounds => {
                if self.draw_bounds {
                    scene.draw_bounds(ctx, pass, batch);
                }
            }
            PassType::Ui => {
                if self.draw_ui {
                    ui.draw(ctx, pass, batch, None, RenderableLifetime::Transient);
                }
            }
            _ => {}
        });

        graph.render(ctx)?;

        graph.end(ctx)?;

        tracing::trace!("End render");

        Ok(())
    }

    pub fn render_ui(
        &mut self,
        ctx: &mut Context,
        graph: &mut FrameGraph,
        ui: &mut UIRenderer,
    ) -> Result<(), RenderError> {
        tracing::trace!("Render frame {}", ctx.frame_number);

        graph.begin(ctx)?;

        graph.prepare(ctx, &mut |pass, batch| {
            if pass.ty() == PassType::Ui {
                ui.draw(ctx, pass, batch, None, RenderableLifetime::Transient);
            }
        });

        graph.render(ctx)?;

        graph.end(ctx)?;

        tracing::trace!("End render");

        Ok(())
    }

    pub fn render_noui(
        &mut self,
        ctx: &mut Context,
        graph: &mut FrameGraph,
        scene: &mut Scene,
    ) -> Result<(), RenderError> {
        tracing::trace!("Render frame {}", ctx.frame_number);

        graph.begin(ctx)?;

        graph.prepare(ctx, &mut |pass, batch| match pass.ty() {
            PassType::Depth | PassType::Forward => {
                scene.draw(ctx, pass, batch, None, RenderableLifetime::Static);
            }
            _ => {}
        });

        graph.render(ctx)?;

        graph.end(ctx)?;

        tracing::trace!("End render");

        Ok(())
    }

    pub fn input(
        &mut self,
        ctx: &Context,
        input: Input,
        graph: &mut FrameGraph,
        scene: &mut Scene,
        ui: &mut UIRenderer,
        camera_controller: Option<&mut CameraController>,
    ) {
        tracing::trace!("Input");

        ui.input(input);
        if let Some(camera_controller) = camera_controller {
            camera_controller.input(input, self.ui.ui_hovered);
        }

        if let Input::KeyPressed(key) = input {
            match key {
                Key::E => graph.render_scaling = (graph.render_scaling + 0.1).min(1.),
                Key::A => graph.render_scaling = (graph.render_scaling - 0.1).max(0.1),
                Key::R => {
                    let rd: Result<RenderDoc<V141>, _> = RenderDoc::new();

                    if let Ok(mut rd) = rd {
                        rd.trigger_capture();
                    }
                }
                Key::L => tracing::info!("{:?}", ctx.device.allocator.allocator.lock().unwrap()),
                Key::P => self.process_updates = !self.process_updates,
                Key::W => self.draw_wire = !self.draw_wire,
                Key::B => self.draw_bounds = !self.draw_bounds,
                Key::U => self.draw_ui = !self.draw_ui,
                Key::O => self.screenshot(ctx, graph),
                Key::Equals => scene.update_camera(|_, camera| {
                    camera.pitch = 0.;
                    camera.yaw = 0.;
                }),
                _ => {}
            }
        }
    }

    pub fn screenshot(&self, ctx: &Context, graph: &mut FrameGraph) {
        let filename = "draw_image.png";
        let mut data = vec![];
        let extent = graph.get_image_data(ctx, "draw", &mut data, ImageFormat::R16g16b16a16Unorm);

        tracing::info!("Screenshot \"{}\" ({} bytes)", filename, data.len());

        let img: ImageBuffer<Rgba<u16>, Vec<u16>> =
            ImageBuffer::from_raw(extent.width, extent.height, data).unwrap();

        img.save_with_format(filename, image::ImageFormat::Png)
            .unwrap();
    }
}

impl Default for SampleApp {
    fn default() -> Self {
        Self::new()
    }
}
