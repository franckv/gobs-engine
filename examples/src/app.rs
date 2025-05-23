use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use renderdoc::{RenderDoc, V141};

use gobs::{
    core::{ImageFormat, Input, Key},
    game::context::GameContext,
    render::{Material, MaterialProperty, Renderable},
    render_graph::{BlendMode, PassType, RenderError},
    resource::{entity::camera::Camera, geometry::VertexAttribute},
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
        tracing::info!(target: "app", "Create");

        Self {
            process_updates: false,
            draw_ui: false,
            draw_bounds: false,
            draw_wire: false,
            ui: Ui::new(),
        }
    }

    pub fn ortho_camera(ctx: &GameContext) -> Camera {
        let extent = ctx.renderer.extent();

        Camera::ortho(extent.width as f32, extent.height as f32, 0.1, 100., 0., 0.)
    }

    pub fn perspective_camera(ctx: &GameContext) -> Camera {
        let extent = ctx.renderer.extent();

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

    pub fn color_material(&self, ctx: &mut GameContext) -> Arc<Material> {
        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;

        Material::builder(&ctx.renderer.gfx, "color.vert.spv", "color.frag.spv")
            .unwrap()
            .vertex_attributes(vertex_attributes)
            .build(
                ctx.renderer.graph.pass_by_type(PassType::Forward).unwrap(),
                &mut ctx.resource_manager,
            )
    }

    pub fn color_material_transparent(&self, ctx: &mut GameContext) -> Arc<Material> {
        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;

        Material::builder(&ctx.renderer.gfx, "color.vert.spv", "color.frag.spv")
            .unwrap()
            .vertex_attributes(vertex_attributes)
            .blend_mode(BlendMode::Alpha)
            .build(
                ctx.renderer.graph.pass_by_type(PassType::Forward).unwrap(),
                &mut ctx.resource_manager,
            )
    }

    pub fn texture_material(&self, ctx: &mut GameContext) -> Arc<Material> {
        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        Material::builder(&ctx.renderer.gfx, "mesh.vert.spv", "mesh.frag.spv")
            .unwrap()
            .vertex_attributes(vertex_attributes)
            .prop("diffuse", MaterialProperty::Texture)
            .build(
                ctx.renderer.graph.pass_by_type(PassType::Forward).unwrap(),
                &mut ctx.resource_manager,
            )
    }

    pub fn texture_material_transparent(&self, ctx: &mut GameContext) -> Arc<Material> {
        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        Material::builder(&ctx.renderer.gfx, "mesh.vert.spv", "mesh.frag.spv")
            .unwrap()
            .vertex_attributes(vertex_attributes)
            .prop("diffuse", MaterialProperty::Texture)
            .blend_mode(BlendMode::Alpha)
            .build(
                ctx.renderer.graph.pass_by_type(PassType::Forward).unwrap(),
                &mut ctx.resource_manager,
            )
    }

    pub fn normal_mapping_material(&self, ctx: &mut GameContext) -> Arc<Material> {
        let vertex_attributes = VertexAttribute::POSITION
            | VertexAttribute::TEXTURE
            | VertexAttribute::NORMAL
            | VertexAttribute::TANGENT
            | VertexAttribute::BITANGENT;

        Material::builder(&ctx.renderer.gfx, "mesh.vert.spv", "mesh_n.frag.spv")
            .unwrap()
            .vertex_attributes(vertex_attributes)
            .prop("diffuse", MaterialProperty::Texture)
            .prop("normal", MaterialProperty::Texture)
            .build(
                ctx.renderer.graph.pass_by_type(PassType::Forward).unwrap(),
                &mut ctx.resource_manager,
            )
    }

    pub fn depth_material(&self, ctx: &mut GameContext) -> Arc<Material> {
        let vertex_attributes = VertexAttribute::POSITION | VertexAttribute::COLOR;

        Material::builder(&ctx.renderer.gfx, "color.vert.spv", "depth.frag.spv")
            .unwrap()
            .vertex_attributes(vertex_attributes)
            .build(
                ctx.renderer.graph.pass_by_type(PassType::Forward).unwrap(),
                &mut ctx.resource_manager,
            )
    }

    pub fn update_ui(
        &mut self,
        ctx: &mut GameContext,
        scene: &Scene,
        ui: &mut UIRenderer,
        delta: f32,
    ) {
        if self.draw_ui {
            let (camera_transform, camera) = scene.camera();

            // TODO: change this
            let app_info = ctx.app_info.clone();

            ui.update(&mut ctx.resource_manager, delta, |ectx| {
                self.ui
                    .draw(&app_info, ectx, scene, camera, &camera_transform);
            });
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut GameContext,
        scene: &mut Scene,
        ui: &mut UIRenderer,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: "app", "Render frame {}", ctx.renderer.frame_number());

        let resource_manager = &mut ctx.resource_manager;

        ctx.renderer.draw(&mut |pass, batch| match pass.ty() {
            PassType::Depth | PassType::Forward => {
                scene.draw(resource_manager, pass, batch, None);
            }
            PassType::Wire => {
                if self.draw_wire {
                    scene.draw(resource_manager, pass, batch, None);
                }
            }
            PassType::Bounds => {
                if self.draw_bounds {
                    scene.draw_bounds(resource_manager, pass, batch);
                }
            }
            PassType::Ui => {
                if self.draw_ui {
                    ui.draw(resource_manager, pass, batch, None);
                }
            }
            _ => {}
        });

        tracing::trace!(target: "app", "End render");

        Ok(())
    }

    pub fn render_ui(
        &mut self,
        ctx: &mut GameContext,
        ui: &mut UIRenderer,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: "app", "Render frame {}", ctx.renderer.frame_number());

        let resource_manager = &mut ctx.resource_manager;

        ctx.renderer.draw(&mut |pass, batch| {
            if pass.ty() == PassType::Ui {
                ui.draw(resource_manager, pass, batch, None);
            }
        });

        tracing::trace!(target: "app", "End render");

        Ok(())
    }

    pub fn render_noui(
        &mut self,
        ctx: &mut GameContext,
        scene: &mut Scene,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: "app", "Render frame {}", ctx.renderer.frame_number());

        let resource_manager = &mut ctx.resource_manager;

        ctx.renderer.draw(&mut |pass, batch| match pass.ty() {
            PassType::Depth | PassType::Forward => {
                scene.draw(resource_manager, pass, batch, None);
            }
            _ => {}
        });

        tracing::trace!(target: "app", "End render");

        Ok(())
    }

    pub fn input(
        &mut self,
        ctx: &mut GameContext,
        input: Input,
        scene: &mut Scene,
        ui: &mut UIRenderer,
        camera_controller: Option<&mut CameraController>,
    ) {
        tracing::trace!(target: "app", "Input");

        ui.input(input);
        if let Some(camera_controller) = camera_controller {
            camera_controller.input(input, self.ui.ui_hovered);
        }

        if let Input::KeyPressed(key) = input {
            match key {
                Key::E => {
                    ctx.renderer.graph.render_scaling =
                        (ctx.renderer.graph.render_scaling + 0.1).min(1.)
                }
                Key::Q => {
                    ctx.renderer.graph.render_scaling =
                        (ctx.renderer.graph.render_scaling - 0.1).max(0.1)
                }
                Key::R => {
                    let rd: Result<RenderDoc<V141>, _> = RenderDoc::new();

                    if let Ok(mut rd) = rd {
                        rd.trigger_capture();
                    }
                }
                Key::L => {
                    tracing::info!(target: "app", "{:?}", ctx.renderer.gfx.device.allocator.allocator.lock().unwrap())
                }
                Key::P => self.process_updates = !self.process_updates,
                Key::Z => self.draw_wire = !self.draw_wire,
                Key::B => self.draw_bounds = !self.draw_bounds,
                Key::U => self.draw_ui = !self.draw_ui,
                Key::O => self.screenshot(ctx),
                Key::Equals => scene.update_camera(|_, camera| {
                    camera.pitch = 0.;
                    camera.yaw = 0.;
                }),
                _ => {}
            }
        }
    }

    pub fn screenshot(&self, ctx: &mut GameContext) {
        let filename = "draw_image.png";
        let mut data = vec![];
        let extent = ctx.renderer.graph.get_image_data(
            &ctx.renderer.gfx,
            "draw",
            &mut data,
            ImageFormat::R16g16b16a16Unorm,
        );

        tracing::info!(target: "app", "Screenshot \"{}\" ({} bytes)", filename, data.len());

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
