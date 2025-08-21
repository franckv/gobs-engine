use image::{ImageBuffer, Rgba};
use renderdoc::{RenderDoc, V141};

use gobs::{
    core::{ImageFormat, Input, Key, logger},
    game::context::GameContext,
    render::{RenderError, Renderable},
    render_graph::PassType,
    resource::entity::camera::Camera,
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
        tracing::info!(target: logger::APP, "Create");

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

    pub fn update_ui(
        &mut self,
        ctx: &mut GameContext,
        scene: &mut Scene,
        ui: &mut UIRenderer,
        delta: f32,
    ) {
        if self.draw_ui {
            // TODO: change this
            // let app_info = ctx.app_info.clone();

            let output = ui.draw_ui(delta, |ectx| {
                self.ui.draw(ectx, ctx, scene, delta);
            });

            ui.update(&mut ctx.resource_manager, output);
        }
    }

    pub fn render(
        &mut self,
        ctx: &mut GameContext,
        scene: Option<&mut Scene>,
        ui: Option<&mut UIRenderer>,
    ) -> Result<(), RenderError> {
        tracing::trace!(target: logger::APP, "Render frame {}", ctx.renderer.frame_number());

        let resource_manager = &mut ctx.resource_manager;

        ctx.renderer
            .draw(resource_manager, &mut |pass, batch, resource_manager| {
                if let Some(scene) = &scene
                    && (self.draw_bounds || !(pass.ty() == PassType::Bounds))
                    && (self.draw_wire || !(pass.ty() == PassType::Wire))
                {
                    scene
                        .draw(resource_manager, pass.clone(), batch, None)
                        .map_err(|_| RenderError::InvalidData)?;
                }
                if let Some(ui) = &ui
                    && self.draw_ui
                {
                    ui.draw(resource_manager, pass, batch, None)
                        .map_err(|_| RenderError::InvalidData)?;
                }

                Ok(())
            })?;

        tracing::trace!(target: logger::APP, "End render");

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
        tracing::trace!(target: logger::APP, "Input");

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

                    match rd {
                        Ok(mut rd) => {
                            rd.trigger_capture();
                        }
                        Err(e) => {
                            tracing::error!("Renderdoc not available: {}", e);
                        }
                    }
                }
                Key::L => {
                    tracing::info!(target: logger::APP, "{:?}", ctx.renderer.gfx.device.allocator.allocator.lock().unwrap())
                }
                Key::P => self.process_updates = !self.process_updates,
                Key::Z => self.draw_wire = !self.draw_wire,
                Key::B => self.draw_bounds = !self.draw_bounds,
                Key::U => self.draw_ui = !self.draw_ui,
                Key::O => self.screenshot(ctx),
                Key::Equals => scene.update_camera(|_, camera| {
                    camera.pitch = 0.;
                    camera.yaw = 0.;

                    true
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

        tracing::info!(target: logger::APP, "Screenshot \"{}\" ({} bytes)", filename, data.len());

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
