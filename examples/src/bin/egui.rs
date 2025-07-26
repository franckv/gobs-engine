use renderdoc::{RenderDoc, V141};

use gobs::{
    core::{Input, Key},
    game::{
        AppError,
        app::{Application, Run},
        context::GameContext,
    },
    render::RenderError,
    ui::UIRenderer,
};

use examples::SampleApp;

struct App {
    common: SampleApp,
    ui: UIRenderer,
    demo: MiscDemoWindow,
}

impl Run for App {
    async fn create(ctx: &mut GameContext) -> Result<Self, AppError> {
        let ui = UIRenderer::new(&ctx.renderer.gfx, &mut ctx.resource_manager, true)?;
        let mut common = SampleApp::new();
        common.draw_ui = true;

        Ok(App {
            common,
            ui,
            demo: Default::default(),
        })
    }

    fn update(&mut self, ctx: &mut GameContext, delta: f32) {
        let output = self
            .ui
            .draw_ui(delta, |ectx| self.demo.show(ectx, &mut true));

        self.ui.update(&mut ctx.resource_manager, output);
    }

    fn render(&mut self, ctx: &mut GameContext) -> Result<(), RenderError> {
        self.common.render(ctx, None, Some(&mut self.ui))
    }

    fn input(&mut self, _ctx: &mut GameContext, input: Input) {
        self.ui.input(input);
        if let Input::KeyPressed(Key::C) = input {
            let rd: Result<RenderDoc<V141>, _> = RenderDoc::new();

            if let Ok(mut rd) = rd {
                rd.trigger_capture();
            }
        }
    }

    fn resize(&mut self, _ctx: &mut GameContext, width: u32, height: u32) {
        self.ui.resize(width, height);
    }

    async fn start(&mut self, _ctx: &mut GameContext) {}

    fn close(&mut self, _ctx: &mut GameContext) {
        tracing::info!(target: "app", "Closed");
    }
}

/// egui misc demo reuse
pub struct MiscDemoWindow {
    num_columns: usize,

    widgets: Widgets,
    colors: ColorWidgets,
    custom_collapsing_header: CustomCollapsingHeader,
    tree: Tree,
    box_painting: BoxPainting,

    dummy_bool: bool,
    dummy_usize: usize,
    checklist: [bool; 3],
}

impl Default for MiscDemoWindow {
    fn default() -> Self {
        Self {
            num_columns: 2,

            widgets: Default::default(),
            colors: Default::default(),
            custom_collapsing_header: Default::default(),
            tree: Tree::demo(),
            box_painting: Default::default(),

            dummy_bool: false,
            dummy_usize: 0,
            checklist: std::array::from_fn(|i| i == 0),
        }
    }
}

impl MiscDemoWindow {
    pub fn name(&self) -> &'static str {
        "✨ Misc Demos"
    }

    pub fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .vscroll(true)
            .hscroll(true)
            .show(ctx, |ui| self.ui(ui));
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.set_min_width(250.0);

        egui::CollapsingHeader::new("Label")
            .default_open(true)
            .show(ui, |ui| {
                label_ui(ui);
            });

        egui::CollapsingHeader::new("Misc widgets")
            .default_open(false)
            .show(ui, |ui| {
                self.widgets.ui(ui);
            });

        egui::CollapsingHeader::new("Text layout")
            .default_open(false)
            .show(ui, |ui| {
                text_layout_demo(ui);
            });

        egui::CollapsingHeader::new("Colors")
            .default_open(false)
            .show(ui, |ui| {
                self.colors.ui(ui);
            });

        egui::CollapsingHeader::new("Custom Collapsing Header")
            .default_open(false)
            .show(ui, |ui| self.custom_collapsing_header.ui(ui));

        egui::CollapsingHeader::new("Tree")
            .default_open(false)
            .show(ui, |ui| self.tree.ui(ui));

        egui::CollapsingHeader::new("Checkboxes")
            .default_open(false)
            .show(ui, |ui| {
                ui.label("Checkboxes with empty labels take up very little space:");
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.horizontal_wrapped(|ui| {
                    for _ in 0..64 {
                        ui.checkbox(&mut self.dummy_bool, "");
                    }
                });
                ui.checkbox(&mut self.dummy_bool, "checkbox");

                ui.label("Radiobuttons are similar:");
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                ui.horizontal_wrapped(|ui| {
                    for i in 0..64 {
                        ui.radio_value(&mut self.dummy_usize, i, "");
                    }
                });
                ui.radio_value(&mut self.dummy_usize, 64, "radio_value");
                ui.label("Checkboxes can be in an indeterminate state:");
                let mut all_checked = self.checklist.iter().all(|item| *item);
                let any_checked = self.checklist.iter().any(|item| *item);
                let indeterminate = any_checked && !all_checked;
                if ui
                    .add(
                        egui::Checkbox::new(&mut all_checked, "Check/uncheck all")
                            .indeterminate(indeterminate),
                    )
                    .changed()
                {
                    self.checklist
                        .iter_mut()
                        .for_each(|checked| *checked = all_checked);
                }
                for (i, checked) in self.checklist.iter_mut().enumerate() {
                    ui.checkbox(checked, format!("Item {}", i + 1));
                }
            });

        ui.collapsing("Columns", |ui| {
            ui.add(egui::Slider::new(&mut self.num_columns, 1..=10).text("Columns"));
            ui.columns(self.num_columns, |cols| {
                for (i, col) in cols.iter_mut().enumerate() {
                    col.label(format!("Column {} out of {}", i + 1, self.num_columns));
                    if i + 1 == self.num_columns && col.button("Delete this").clicked() {
                        self.num_columns -= 1;
                    }
                }
            });
        });

        egui::CollapsingHeader::new("Test box rendering")
            .default_open(false)
            .show(ui, |ui| self.box_painting.ui(ui));

        egui::CollapsingHeader::new("Resize")
            .default_open(false)
            .show(ui, |ui| {
                egui::Resize::default()
                    .default_height(100.0)
                    .show(ui, |ui| {
                        ui.label("This ui can be resized!");
                        ui.label("Just pull the handle on the bottom right");
                    });
            });

        egui::CollapsingHeader::new("Misc")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("You can pretty easily paint your own small icons:");
                    use std::f32::consts::TAU;
                    let size = egui::Vec2::splat(16.0);
                    let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
                    let rect = response.rect;
                    let c = rect.center();
                    let r = rect.width() / 2.0 - 1.0;
                    let color = egui::Color32::from_gray(128);
                    let stroke = egui::Stroke::new(1.0, color);
                    painter.circle_stroke(c, r, stroke);
                    painter.line_segment([c - egui::vec2(0.0, r), c + egui::vec2(0.0, r)], stroke);
                    painter.line_segment([c, c + r * egui::Vec2::angled(TAU * 1.0 / 8.0)], stroke);
                    painter.line_segment([c, c + r * egui::Vec2::angled(TAU * 3.0 / 8.0)], stroke);
                });
            });

        egui::CollapsingHeader::new("Many circles of different sizes")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for i in 0..100 {
                        let r = i as f32 * 0.5;
                        let size = egui::Vec2::splat(2.0 * r + 5.0);
                        let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), r, ui.visuals().text_color());
                    }
                });
            });
    }
}

// ----------------------------------------------------------------------------

fn label_ui(ui: &mut egui::Ui) {
    ui.horizontal_wrapped(|ui| {
        // Trick so we don't have to add spaces in the text below:
        let width = ui.fonts(|f|f.glyph_width(&egui::TextStyle::Body.resolve(ui.style()), ' '));
        ui.spacing_mut().item_spacing.x = width;

        ui.label(egui::RichText::new("Text can have").color(egui::Color32::from_rgb(110, 255, 110)));
        ui.colored_label(egui::Color32::from_rgb(128, 140, 255), "color"); // Shortcut version
        ui.label("and tooltips.").on_hover_text(
            "This is a multiline tooltip that demonstrates that you can easily add tooltips to any element.\nThis is the second line.\nThis is the third.",
        );

        ui.label("You can mix in other widgets into text, like");
        let _ = ui.small_button("this button");
        ui.label(".");

        ui.label("The default font supports all latin and cyrillic characters (ИÅđ…), common math symbols (∫√∞²⅓…), and many emojis (💓🌟🖩…).")
            .on_hover_text("There is currently no support for right-to-left languages.");
        ui.label("See the 🔤 Font Book for more!");

        ui.monospace("There is also a monospace font.");
    });

    ui.add(
        egui::Label::new(
            "Labels containing long text can be set to elide the text that doesn't fit on a single line using `Label::truncate`. When hovered, the label will show the full text.",
        )
            .truncate(),
    );
}

// ----------------------------------------------------------------------------

pub struct Widgets {
    angle: f32,
    _password: String,
}

impl Default for Widgets {
    fn default() -> Self {
        Self {
            angle: std::f32::consts::TAU / 3.0,
            _password: "hunter2".to_owned(),
        }
    }
}

impl Widgets {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { angle, _password } = self;
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("An angle:");
            ui.drag_angle(angle);
            ui.label(format!("≈ {:.3}τ", *angle / std::f32::consts::TAU))
                .on_hover_text("Each τ represents one turn (τ = 2π)");
        })
        .response
        .on_hover_text("The angle is stored in radians, but presented in degrees");

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Password:")
                .on_hover_text("See the example code for how to use egui to store UI state");
        });
    }
}

// ----------------------------------------------------------------------------

#[derive(PartialEq)]
struct ColorWidgets {
    srgba_unmul: [u8; 4],
    srgba_premul: [u8; 4],
    rgba_unmul: [f32; 4],
    rgba_premul: [f32; 4],
}

impl Default for ColorWidgets {
    fn default() -> Self {
        // Approximately the same color.
        Self {
            srgba_unmul: [0, 255, 183, 127],
            srgba_premul: [0, 187, 140, 127],
            rgba_unmul: [0.0, 1.0, 0.5, 0.5],
            rgba_premul: [0.0, 0.5, 0.25, 0.5],
        }
    }
}

impl ColorWidgets {
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::reset_button(ui, self, "Reset");

        ui.label("egui lets you edit colors stored as either sRGBA or linear RGBA and with or without premultiplied alpha");

        let Self {
            srgba_unmul,
            srgba_premul,
            rgba_unmul,
            rgba_premul,
        } = self;

        ui.horizontal(|ui| {
            ui.color_edit_button_srgba_unmultiplied(srgba_unmul);
            ui.label(format!(
                "sRGBA: {} {} {} {}",
                srgba_unmul[0], srgba_unmul[1], srgba_unmul[2], srgba_unmul[3],
            ));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_srgba_premultiplied(srgba_premul);
            ui.label(format!(
                "sRGBA with premultiplied alpha: {} {} {} {}",
                srgba_premul[0], srgba_premul[1], srgba_premul[2], srgba_premul[3],
            ));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_unmultiplied(rgba_unmul);
            ui.label(format!(
                "Linear RGBA: {:.02} {:.02} {:.02} {:.02}",
                rgba_unmul[0], rgba_unmul[1], rgba_unmul[2], rgba_unmul[3],
            ));
        });

        ui.horizontal(|ui| {
            ui.color_edit_button_rgba_premultiplied(rgba_premul);
            ui.label(format!(
                "Linear RGBA with premultiplied alpha: {:.02} {:.02} {:.02} {:.02}",
                rgba_premul[0], rgba_premul[1], rgba_premul[2], rgba_premul[3],
            ));
        });
    }
}

// ----------------------------------------------------------------------------

struct BoxPainting {
    size: egui::Vec2,
    rounding: f32,
    stroke_width: f32,
    num_boxes: usize,
}

impl Default for BoxPainting {
    fn default() -> Self {
        Self {
            size: egui::vec2(64.0, 32.0),
            rounding: 5.0,
            stroke_width: 2.0,
            num_boxes: 1,
        }
    }
}

impl BoxPainting {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.size.x, 0.0..=500.0).text("width"));
        ui.add(egui::Slider::new(&mut self.size.y, 0.0..=500.0).text("height"));
        ui.add(egui::Slider::new(&mut self.rounding, 0.0..=50.0).text("rounding"));
        ui.add(egui::Slider::new(&mut self.stroke_width, 0.0..=10.0).text("stroke_width"));
        ui.add(egui::Slider::new(&mut self.num_boxes, 0..=8).text("num_boxes"));

        ui.horizontal_wrapped(|ui| {
            for _ in 0..self.num_boxes {
                let (rect, _response) = ui.allocate_at_least(self.size, egui::Sense::hover());
                ui.painter().rect(
                    rect,
                    self.rounding,
                    ui.visuals().text_color().gamma_multiply(0.5),
                    egui::Stroke::new(self.stroke_width, egui::Color32::WHITE),
                    egui::StrokeKind::Inside,
                );
            }
        });
    }
}

// ----------------------------------------------------------------------------

struct CustomCollapsingHeader {
    selected: bool,
    radio_value: bool,
}

impl Default for CustomCollapsingHeader {
    fn default() -> Self {
        Self {
            selected: true,
            radio_value: false,
        }
    }
}

impl CustomCollapsingHeader {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Example of a collapsing header with custom header:");

        let id = ui.make_persistent_id("my_collapsing_header");
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui| {
                ui.toggle_value(&mut self.selected, "Click to select/unselect");
                ui.radio_value(&mut self.radio_value, false, "");
                ui.radio_value(&mut self.radio_value, true, "");
            })
            .body(|ui| {
                ui.label("The body is always custom");
            });

        egui::CollapsingHeader::new("Normal collapsing header for comparison").show(ui, |ui| {
            ui.label("Nothing exciting here");
        });
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Action {
    Keep,
    Delete,
}

#[derive(Clone, Default)]
struct Tree(Vec<Tree>);

impl Tree {
    pub fn demo() -> Self {
        Self(vec![
            Self(vec![Self::default(); 4]),
            Self(vec![Self(vec![Self::default(); 2]); 3]),
        ])
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> Action {
        self.ui_impl(ui, 0, "root")
    }
}

impl Tree {
    fn ui_impl(&mut self, ui: &mut egui::Ui, depth: usize, name: &str) -> Action {
        egui::CollapsingHeader::new(name)
            .default_open(depth < 1)
            .show(ui, |ui| self.children_ui(ui, depth))
            .body_returned
            .unwrap_or(Action::Keep)
    }

    fn children_ui(&mut self, ui: &mut egui::Ui, depth: usize) -> Action {
        if depth > 0
            && ui
                .button(egui::RichText::new("delete").color(ui.visuals().warn_fg_color))
                .clicked()
        {
            return Action::Delete;
        }

        self.0 = std::mem::take(self)
            .0
            .into_iter()
            .enumerate()
            .filter_map(|(i, mut tree)| {
                if tree.ui_impl(ui, depth + 1, &format!("child #{i}")) == Action::Keep {
                    Some(tree)
                } else {
                    None
                }
            })
            .collect();

        if ui.button("+").clicked() {
            self.0.push(Self::default());
        }

        Action::Keep
    }
}

// ----------------------------------------------------------------------------

fn text_layout_demo(ui: &mut egui::Ui) {
    use egui::text::LayoutJob;

    let mut job = LayoutJob::default();

    let first_row_indentation = 10.0;

    let (default_color, strong_color) = if ui.visuals().dark_mode {
        (egui::Color32::LIGHT_GRAY, egui::Color32::WHITE)
    } else {
        (egui::Color32::DARK_GRAY, egui::Color32::BLACK)
    };

    job.append(
        "This is a demonstration of ",
        first_row_indentation,
        egui::TextFormat {
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "the egui text layout engine. ",
        0.0,
        egui::TextFormat {
            color: strong_color,
            ..Default::default()
        },
    );
    job.append(
        "It supports ",
        0.0,
        egui::TextFormat {
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "different ",
        0.0,
        egui::TextFormat {
            color: egui::Color32::from_rgb(110, 255, 110),
            ..Default::default()
        },
    );
    job.append(
        "colors, ",
        0.0,
        egui::TextFormat {
            color: egui::Color32::from_rgb(128, 140, 255),
            ..Default::default()
        },
    );
    job.append(
        "backgrounds, ",
        0.0,
        egui::TextFormat {
            color: default_color,
            background: egui::Color32::from_rgb(128, 32, 32),
            ..Default::default()
        },
    );
    job.append(
        "mixing ",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(17.0),
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "fonts, ",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(12.0),
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "raised text, ",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(7.0),
            color: default_color,
            valign: egui::Align::TOP,
            ..Default::default()
        },
    );
    job.append(
        "with ",
        0.0,
        egui::TextFormat {
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "underlining",
        0.0,
        egui::TextFormat {
            color: default_color,
            underline: egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
            ..Default::default()
        },
    );
    job.append(
        " and ",
        0.0,
        egui::TextFormat {
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "strikethrough",
        0.0,
        egui::TextFormat {
            color: default_color,
            strikethrough: egui::Stroke::new(2.0, egui::Color32::RED.linear_multiply(0.5)),
            ..Default::default()
        },
    );
    job.append(
        ". Of course, ",
        0.0,
        egui::TextFormat {
            color: default_color,
            ..Default::default()
        },
    );
    job.append(
        "you can",
        0.0,
        egui::TextFormat {
            color: default_color,
            strikethrough: egui::Stroke::new(1.0, strong_color),
            ..Default::default()
        },
    );
    job.append(
        " mix these!",
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(7.0),
            color: egui::Color32::LIGHT_BLUE,
            background: egui::Color32::from_rgb(128, 0, 0),
            underline: egui::Stroke::new(1.0, strong_color),
            ..Default::default()
        },
    );

    ui.label(job);
}

fn main() {
    examples::init_logger();

    tracing::info!(target: "app", "Engine start");

    Application::<App>::new("Egui", examples::WIDTH, examples::HEIGHT).run();
}
