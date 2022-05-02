use crate::syntax_highlighting::CodeTheme;
use eframe::{egui, egui::FontData, egui::FontDefinitions, egui::FontFamily, epi};

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

    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Option<String>,
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
            code: "// Time to write some code...\n\
                    fn main() {}"
                .into(),
            dropped_files: Vec::new(),
            picked_path: None,
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
        self.setup_custom_font(_ctx);
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
            dropped_files,
            picked_path,
        } = self;

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
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.picked_path = Some(path.display().to_string());
                            println!("Open file {:?}", self.picked_path);
                        }
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
            *dropped_files = ctx.input().raw.dropped_files.clone();
            println!("Dropped files: {:?}", *dropped_files)
        }
    }
}
