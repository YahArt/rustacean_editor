use crate::syntax_highlighting::CodeTheme;
use eframe::{egui, epi};
use std::collections::BTreeMap;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    file_name: String,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    row: i32,
    col: i32,

    language: String,
    code: String,
    font_size: i32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            file_name: "[empty]".to_owned(),
            row: 0,
            col: 0,
            language: "rs".into(),
            font_size: 32,
            code: "// A very simple example\n\
                    fn main() {\n\
\tprintln!(\"Hello world!\");\n\
}\n\
"
            .into(),
        }
    }
}

impl epi::App for TemplateApp {
    fn name(&self) -> &str {
        "Rustacean Editor"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {
            file_name,
            row,
            col,
            language,
            code,
            font_size,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                if ui.button("Quit").clicked() {
                    frame.quit();
                }
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        println!("File Open clicked...");
                    }
                    if ui.button("Save").clicked() {
                        println!("File Save clicked...");
                    }
                });

                if ui.button("About").clicked() {
                    println!("About clicked...");
                }
            });

            ui.horizontal(|ui| {
                let row_message = format!("Row: {}", row);
                let col_message = format!("Col: {}", col);

                ui.label(file_name.to_owned());
                ui.label(row_message.to_owned());
                ui.label(col_message.to_owned());
            });

            // Toggle buttons for themes...
            eframe::egui::widgets::global_dark_light_mode_buttons(ui);

            // Add slider for changing font size
            ui.add(egui::Slider::new(font_size, 16..=64).prefix("Font Size: "));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let theme = CodeTheme::from_memory(ui.ctx());
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = crate::syntax_highlighting::highlight(
                    ui.ctx(),
                    &theme,
                    string,
                    language,
                    font_size,
                );
                layout_job.wrap_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            };
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        // TODO: Make rows dependent on number of lines in code file...
                        .desired_rows(20)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
            egui::warn_if_debug_build(ui);
        });
    }
}
