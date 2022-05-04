use crate::syntax_highlighting::CodeTheme;
use eframe::{egui, egui::FontData, egui::FontDefinitions, egui::FontFamily, epi};
use std::fs;
use std::path::PathBuf;

const SUPPORTED_FONT_SIZES: [i32; 5] = [16, 18, 20, 22, 24];

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    code: String,

    language: String,
    font_size: i32,
    file_name: String,
    is_dark_theme: bool,

    #[serde(skip)]
    theme: CodeTheme,
}

impl TemplateApp {
    fn setup_custom_font(&mut self, _ctx: &egui::Context) {
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
        match file_path {
            Some(file_path) => {
                self.file_name = file_path.display().to_string();
                self.code = fs::read_to_string(self.file_name.clone())
                    .expect("Something went wrong reading the file");
            }
            None => println!("No valid file path provided..."),
        }
    }

    fn apply_theme(&mut self, _ctx: &egui::Context) {
        if self.is_dark_theme {
            self.theme = CodeTheme::dark();
            _ctx.set_visuals(egui::Visuals::dark());
        } else {
            self.theme = CodeTheme::light();
            _ctx.set_visuals(egui::Visuals::light());
        }
    }

    // On the web target we do not support any file related things
    #[cfg(not(target_arch = "wasm32"))]
    fn save_file(&mut self, file_path: Option<PathBuf>) {
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
            font_size: SUPPORTED_FONT_SIZES[0],
            code: "// Time to write some code...\n\
                    fn main() {}"
                .into(),
            is_dark_theme: true,
            theme: CodeTheme::default(),
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
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default();
        }
        self.apply_theme(_ctx);
        self.setup_custom_font(_ctx);
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
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

                ui.menu_button("Config", |ui| {
                    ui.menu_button("Font Size", |ui| {
                        for font_size in SUPPORTED_FONT_SIZES {
                            if ui.button(format!("Font Size:{}", font_size)).clicked() {
                                self.font_size = font_size;
                            }
                        }
                    });

                    ui.menu_button("Themes", |ui| {
                        if ui
                            .radio_value(&mut self.is_dark_theme, true, "Dark Theme")
                            .clicked()
                        {
                            self.apply_theme(ui.ctx());
                        }

                        if ui
                            .radio_value(&mut self.is_dark_theme, false, "Light Theme")
                            .clicked()
                        {
                            self.apply_theme(ui.ctx());
                        }
                    });
                });
            });

            ui.horizontal(|ui| {
                ui.label("Current File: ");
                ui.label(self.file_name.clone());
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                let mut layout_job = crate::syntax_highlighting::highlight(
                    ui.ctx(),
                    &self.theme,
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
