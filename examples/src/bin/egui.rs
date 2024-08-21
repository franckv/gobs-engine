use gobs::{
    game::{
        app::{Application, Run},
        input::{Input, Key},
    },
    gfx::Device,
    render::{
        context::Context,
        graph::{FrameGraph, RenderError},
        pass::PassType,
        renderable::Renderable,
    },
    ui::UIRenderer,
};
use renderdoc::{RenderDoc, V141};

struct App {
    graph: FrameGraph,
    ui: UIRenderer,
}

impl Run for App {
    async fn create(ctx: &Context) -> Self {
        let graph = FrameGraph::ui(ctx);
        let ui = UIRenderer::new(ctx, graph.pass_by_type(PassType::Ui).unwrap());

        App { graph, ui }
    }

    fn update(&mut self, ctx: &Context, delta: f32) {
        self.graph.update(ctx, delta);

        self.ui.update(
            ctx,
            self.graph.pass_by_type(PassType::Ui).unwrap(),
            delta,
            |ectx| {
                egui::Window::new("egui").show(ectx, |ui| {
                    ui.visuals_mut().override_text_color = Some(egui::Color32::GREEN);
                    ectx.style_mut(|s| {
                        for (_, id) in s.text_styles.iter_mut() {
                            id.size = 16.;
                        }
                    });
                    ui.heading("Header");
                    ui.separator();
                    ui.collapsing("Some content", |ui| {
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for _ in 0..100 {
                                ui.label("Content");
                            }
                        });
                    });
                });
            },
        );
    }

    fn render(&mut self, ctx: &mut Context) -> Result<(), RenderError> {
        tracing::trace!("Render frame {}", ctx.frame_number);

        self.graph.begin(ctx)?;

        self.graph.render(ctx, &mut |pass, batch| match pass.ty() {
            PassType::Ui => {
                self.ui.draw(ctx, pass, batch);
            }
            _ => (),
        })?;

        self.graph.end(ctx)?;

        tracing::trace!("End render");

        Ok(())
    }

    fn input(&mut self, ctx: &Context, input: Input) {
        self.ui.input(input);
        match input {
            Input::KeyPressed(key) => match key {
                Key::C => {
                    let rd: Result<RenderDoc<V141>, _> = RenderDoc::new();

                    if let Ok(mut rd) = rd {
                        rd.trigger_capture();
                    }

                    self.ui.dump_model(ctx);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn resize(&mut self, ctx: &mut Context, width: u32, height: u32) {
        self.graph.resize(ctx);
        self.ui.resize(width, height);
    }

    async fn start(&mut self, _ctx: &Context) {}

    fn close(&mut self, ctx: &Context) {
        tracing::info!("Closing");

        ctx.device.wait();

        tracing::info!("Closed");
    }
}

fn main() {
    examples::init_logger();

    tracing::info!("Engine start");

    Application::<App>::new("Egui", examples::WIDTH, examples::HEIGHT).run();
}
