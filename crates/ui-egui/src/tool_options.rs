//! The Tool Options bar (Window › Tool Options): a strip under the Control bar showing the tool in
//! hand and its settings, changing as tools change: the Pencil's and Paintbrush's Fidelity and
//! tolerances, the Blob Brush's size, the shape tools' corners, sides and points, the line tools'
//! segments and dividers, the Liquify and Symbolism tools' brush. They are the settings the tools'
//! option dialogs (double-click a tool) set, through `tool.setOption`. With it on, the Control bar
//! leaves the tool settings it shows for a few tools (Artboard, Crop Image, Puppet Warp, Mirror &
//! Cut) to it.

use egui::{Sense, Stroke, Ui, vec2};
use serde_json::{Value, json};

use crate::theme::{self, Tokens};
use crate::{VectorcraftApp, icons, widgets};

/// The Liquify tools, which share a brush.
const LIQUIFY: [&str; 7] = ["warp", "twirl", "pucker", "bloat", "scallop", "crystallize", "wrinkle"];

/// Height of the bar.
pub const HEIGHT: f32 = 34.0;

/// The Tool Options bar, under the Control bar.
pub fn show(app: &mut VectorcraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    egui::Panel::top("tool_options_bar")
        .exact_size(HEIGHT)
        .frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(10, 0)).stroke(Stroke::new(1.0, t.border)))
        .show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let tool = app.session.tool_id().to_string();
                let info = vectorcraft_tools::tool_info(&tool);
                if let Some(info) = info {
                    let (r, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                    icons::paint(ui, info.icon, r, t.text);
                    ui.add_space(4.0);
                }
                let name = info.map_or(tool.as_str(), |i| i.label);
                ui.label(egui::RichText::new(tl!(name)).font(theme::semibold(12.0)).color(t.text));
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(4.0);
                if !fields(app, ui, &tool) {
                    ui.label(egui::RichText::new(tl!("This tool has no options")).size(12.0).color(t.text_dim));
                }
            });
        });
}

/// A number setting: `key`, shown `scale` times its value with `suffix`, set back divided by it;
/// `whole` sends whole numbers (counts).
struct Num<'a> {
    key: &'a str,
    label: &'a str,
    suffix: &'a str,
    decimals: usize,
    step: f64,
    min: f64,
    presets: &'a [f64],
    scale: f64,
    whole: bool,
}

impl<'a> Num<'a> {
    fn new(key: &'a str, label: &'a str, suffix: &'a str) -> Self {
        Self { key, label, suffix, decimals: 0, step: 1.0, min: 0.0, presets: &[], scale: 1.0, whole: false }
    }
    fn decimals(self, decimals: usize) -> Self {
        Self { decimals, ..self }
    }
    fn step(self, step: f64, min: f64) -> Self {
        Self { step, min, ..self }
    }
    fn presets(self, presets: &'a [f64]) -> Self {
        Self { presets, ..self }
    }
    fn percent(self) -> Self {
        Self { scale: 100.0, ..self }
    }
    fn whole(self) -> Self {
        Self { whole: true, ..self }
    }
}

/// Set the tool's option `key`; a failure (none expected) shows in the status bar.
fn set(app: &mut VectorcraftApp, key: &str, value: Value) {
    if let Err(e) = app.run("tool.setOption", json!({ "key": key, "value": value })) {
        app.ui.status = e;
    }
}

/// A labelled number field for the tool's option.
fn number(app: &mut VectorcraftApp, ui: &mut Ui, opts: &Value, n: Num) {
    let t = Tokens::get(ui.ctx());
    ui.label(egui::RichText::new(n.label).size(12.0).color(t.text));
    let value = opts.get(n.key).and_then(Value::as_f64).filter(|v| v.is_finite()).map(|v| v * n.scale);
    let picked = widgets::spin_plain(ui, ("tool-option", n.key), value, n.suffix, n.decimals, 70.0, n.step, n.min, n.presets);
    if let Some(v) = picked.filter(|v| v.is_finite()) {
        let v = v / n.scale;
        // Counts go as whole numbers: the tools read them as integers.
        let value = if n.whole { json!(v.round().clamp(0.0, 1e6) as u64) } else { json!(v) };
        set(app, n.key, value);
    }
    ui.add_space(10.0);
}

/// A checkbox for the tool's option `key`.
fn check(app: &mut VectorcraftApp, ui: &mut Ui, opts: &Value, key: &str, label: &str) {
    let on = opts.get(key).and_then(Value::as_bool).unwrap_or(false);
    if widgets::check(ui, label, on, true) {
        set(app, key, json!(!on));
    }
    ui.add_space(10.0);
}

/// The settings of `tool`; false when it has none.
fn fields(app: &mut VectorcraftApp, ui: &mut Ui, tool: &str) -> bool {
    let opts = app.session.tool_options();
    let o = &opts;
    match tool {
        "pencil" | "paintbrush" => {
            number(app, ui, o, Num::new("fidelity", tl!("Fidelity:"), "").decimals(1).step(0.5, 0.05).presets(&[0.5, 1.0, 2.5, 5.0, 10.0, 20.0]));
            check(app, ui, o, "fill", tl!("Fill new strokes"));
            number(app, ui, o, Num::new("editWithin", tl!("Edit selected paths within:"), " px"));
            number(app, ui, o, Num::new("closeWithin", tl!("Close paths when ends are within:"), " px"));
        }
        "smooth" => {
            number(app, ui, o, Num::new("fidelity", tl!("Fidelity:"), "").decimals(1).step(0.5, 0.05).presets(&[0.5, 1.0, 2.5, 5.0, 10.0, 20.0]))
        }
        "blobBrush" | "eraser" => {
            number(app, ui, o, Num::new("size", tl!("Size:"), " pt").decimals(1).step(1.0, 0.1).presets(&[1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0]))
        }
        "roundedRectangle" => {
            number(app, ui, o, Num::new("cornerRadius", tl!("Corner Radius:"), " pt").decimals(1).presets(&[0.0, 4.0, 8.0, 12.0, 24.0]))
        }
        "polygon" => number(app, ui, o, Num::new("sides", tl!("Sides:"), "").step(1.0, 3.0).whole().presets(&[3.0, 4.0, 5.0, 6.0, 8.0, 12.0])),
        "star" => {
            number(app, ui, o, Num::new("points", tl!("Points:"), "").step(1.0, 3.0).whole().presets(&[3.0, 4.0, 5.0, 6.0, 8.0, 12.0]));
            number(app, ui, o, Num::new("starRatio", tl!("Inner Radius:"), " %").step(5.0, 1.0).percent().presets(&[25.0, 38.0, 50.0, 75.0]));
        }
        "spiral" => {
            number(app, ui, o, Num::new("decay", tl!("Decay:"), " %").step(5.0, 5.0).presets(&[50.0, 75.0, 80.0, 90.0, 95.0]));
            number(app, ui, o, Num::new("segments", tl!("Segments:"), "").step(1.0, 2.0).whole());
            check(app, ui, o, "clockwise", tl!("Clockwise"));
        }
        "rectangularGrid" => {
            number(app, ui, o, Num::new("rows", tl!("Rows:"), "").whole());
            number(app, ui, o, Num::new("columns", tl!("Columns:"), "").whole());
        }
        "polarGrid" => {
            number(app, ui, o, Num::new("concentric", tl!("Concentric Dividers:"), "").whole());
            number(app, ui, o, Num::new("radial", tl!("Radial Dividers:"), "").whole());
        }
        "arc" => check(app, ui, o, "closed", tl!("Closed")),
        _ if tool.starts_with("symbol") => {
            number(app, ui, o, Num::new("diameter", tl!("Diameter:"), " pt").step(10.0, 2.0).presets(&[20.0, 50.0, 100.0, 200.0, 400.0]));
            number(app, ui, o, Num::new("intensity", tl!("Intensity:"), "").step(1.0, 1.0));
            number(app, ui, o, Num::new("density", tl!("Density:"), "").step(1.0, 1.0));
        }
        _ if LIQUIFY.contains(&tool) => {
            number(app, ui, o, Num::new("width", tl!("Width:"), " pt").step(5.0, 0.5).presets(&[20.0, 50.0, 100.0, 200.0]));
            number(app, ui, o, Num::new("height", tl!("Height:"), " pt").step(5.0, 0.5).presets(&[20.0, 50.0, 100.0, 200.0]));
            number(app, ui, o, Num::new("angle", tl!("Angle:"), "°").step(15.0, -360.0));
            number(app, ui, o, Num::new("intensity", tl!("Intensity:"), " %").step(5.0, 0.0).percent().presets(&[10.0, 25.0, 50.0, 75.0, 100.0]));
        }
        // Tools whose settings the Control bar has shown: the same controls.
        "artboard" | "puppetWarp" | "mirrorCut" => crate::toolbar::control_bar_options(app, ui),
        _ if tool == vectorcraft_tools::cropimage::ID => crate::toolbar::control_bar_options(app, ui),
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vectorcraft_engine::Session;

    use crate::VectorcraftApp;

    /// The bar's text after a few frames, 1400 points wide, with the tool `tool` in hand.
    fn bar_text(app: &mut VectorcraftApp, tool: &str) -> String {
        app.run("tool.select", json!({ "tool": tool })).unwrap();
        let ctx = egui::Context::default();
        let mut text = String::new();
        for _ in 0..3 {
            let raw =
                egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1400.0, 900.0))), ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| {
                app.logic(ui.ctx());
                app.ui(ui);
            });
            out.textures_delta.clear();
            text = out
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Text(t) => Some(t.galley.text().to_string()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("|");
        }
        text
    }

    #[test]
    fn the_bar_shows_the_tool_in_hand_and_its_settings() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("file.new", json!({"width": 300, "height": 300})).unwrap();
        assert!(!app.ui.tool_options, "off by default");
        app.run("window.toolOptions", json!({})).unwrap();
        assert!(app.ui.tool_options);
        assert_eq!(crate::menus::checked(&app, "window.toolOptions", &json!({})), Some(true));
        let text = bar_text(&mut app, "pencil");
        assert!(text.contains("Pencil Tool") && text.contains("Fidelity:") && text.contains("Fill new strokes"), "{text}");
        let text = bar_text(&mut app, "star");
        assert!(text.contains("Star Tool") && text.contains("Points:") && text.contains("Inner Radius:"), "{text}");
        let text = bar_text(&mut app, "warp");
        assert!(text.contains("Width:") && text.contains("Intensity:"), "{text}");
        let text = bar_text(&mut app, "hand");
        assert!(text.contains("Hand Tool") && text.contains("This tool has no options"), "{text}");
        // Kept in workspaces.
        app.run("window.workspace.new", json!({"name": "Options"})).unwrap();
        app.run("window.workspace", json!({"name": "Essentials"})).unwrap();
        assert!(!app.ui.tool_options);
        app.run("window.workspace", json!({"name": "Options"})).unwrap();
        assert!(app.ui.tool_options);
    }

    #[test]
    fn counts_reach_the_tools_as_whole_numbers() {
        let mut app = VectorcraftApp::new(Session::new(), Default::default());
        app.run("tool.select", json!({"tool": "polygon"})).unwrap();
        super::set(&mut app, "sides", json!(8));
        assert_eq!(app.session.tool_options()["sides"], json!(8));
        // A float is no count to the tool, which falls back to its default: why the bar sends
        // counts whole.
        super::set(&mut app, "sides", json!(9.0));
        assert_eq!(app.session.tool_options()["sides"], json!(6));
    }
}
