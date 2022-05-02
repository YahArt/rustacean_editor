use crate::syntax_highlighting::CodeTheme;
use eframe::{egui, egui::FontData, egui::FontDefinitions, egui::FontFamily, epi};
use std::fs;
use std::path::PathBuf;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    language: String,
    code: String,
    font_size: i32,
    file_name: String,
}

impl TemplateApp {
    fn setup_custom_font(&self, _ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            FontData::from_static(include_bytes!("fonts/FiraCode-Regular.ttf")), // .ttf and .otf supported
        );

        // Put custom font first for proportional and monospace fonts (highest priority):
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "custom_font".to_owned());

        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "custom_font".to_owned());
        _ctx.set_fonts(fonts);
    }

    fn read_file(&mut self, file_path: Option<PathBuf>) {
        // Only allow certain file types because of reasons...
        println!("Load file {:?}", file_path);
        match file_path {
            Some(file_path) => {
                self.file_name = file_path.display().to_string();
                self.code = fs::read_to_string(self.file_name.clone())
                    .expect("Something went wrong reading the file");
            }
            None => println!("No valid file path provided..."),
        }
    }

    fn save_file(&mut self, file_path: Option<PathBuf>) {
        // Only allow certain file types because of reasons...
        println!("Save file {:?}", file_path);
        match file_path {
            Some(file_path) => {
                let file_name = file_path.display().to_string();
                let file_content = self.code.clone();
                fs::write(file_name, file_content)
                    .expect("Something wrent wrong while saving file");
            }
            None => println!("No valid file path provided..."),
        }
    }
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            file_name: String::from("[empty]"),
            language: String::from("rs"),
            font_size: 32,
            code: "// Time to write some code...\n\
                    fn main() {}"
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
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
        self.setup_custom_font(_ctx);
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                if ui.button("Quit").clicked() {
                    frame.quit();
                }

                // On the web target we do not support any file related things
                #[cfg(not(target_arch = "wasm32"))]
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        self.read_file(
                            rfd::FileDialog::new()
                                .add_filter("rust", &["rs"])
                                .pick_file(),
                        );
                    }
                    if ui.button("Save").clicked() {
                        self.save_file(
                            rfd::FileDialog::new()
                                .add_filter("rust", &["rs"])
                                .save_file(),
                        )
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.label(self.file_name.clone());
            });

            // Toggle buttons for themes...
            eframe::egui::widgets::global_dark_light_mode_buttons(ui);

            // Add slider for changing font size
            ui.add(egui::Slider::new(&mut self.font_size, 16..=64).prefix("Font Size: "));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let theme = CodeTheme::from_memory(ui.ctx());
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = crate::syntax_highlighting::highlight(
                    ui.ctx(),
                    &theme,
                    string,
                    &self.language.clone(),
                    &self.font_size.clone(),
                );
                layout_job.wrap_width = wrap_width;
                ui.fonts().layout_job(layout_job)
            };
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.code)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(20)
                        .lock_focus(true)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
            egui::warn_if_debug_build(ui);
        });

        // Collect dropped files:
        if !ctx.input().raw.dropped_files.is_empty() {
            let dropped_files = ctx.input().raw.dropped_files.clone();
            self.read_file(dropped_files.first().unwrap().path.clone())
        }
    }
}
