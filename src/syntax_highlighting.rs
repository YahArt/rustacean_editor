use eframe::egui::text::LayoutJob;

/// Memoized Code highlighting
pub fn highlight(
    ctx: &eframe::egui::Context,
    theme: &CodeTheme,
    code: &str,
    language: &str,
    font_size: &i32,
) -> LayoutJob {
    impl eframe::egui::util::cache::ComputerMut<(&CodeTheme, &str, &str, &i32), LayoutJob>
        for Highlighter
    {
        fn compute(
            &mut self,
            (theme, code, lang, font_size): (&CodeTheme, &str, &str, &i32),
        ) -> LayoutJob {
            self.highlight(theme, code, lang, font_size)
        }
    }

    type HighlightCache<'a> = eframe::egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    let mut memory = ctx.memory();
    let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
    highlight_cache.get((theme, code, language, font_size))
}

// ----------------------------------------------------------------------------
#[derive(Clone, Copy, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
enum SyntectTheme {
    Base16MochaDark,
    SolarizedLight,
}

impl SyntectTheme {
    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::SolarizedLight => "Solarized (light)",
        }
    }
}

#[derive(Clone, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeTheme {
    dark_mode: bool,
    syntect_theme: SyntectTheme,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    pub fn dark() -> Self {
        Self {
            dark_mode: true,
            syntect_theme: SyntectTheme::Base16MochaDark,
        }
    }

    pub fn light() -> Self {
        Self {
            dark_mode: false,
            syntect_theme: SyntectTheme::SolarizedLight,
        }
    }
}

// ----------------------------------------------------------------------------

struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, code: &str, lang: &str, font_size: &i32) -> LayoutJob {
        self.highlight_impl(theme, code, lang, font_size)
            .unwrap_or_else(|| {
                // Fallback:
                LayoutJob::simple(
                    code.into(),
                    eframe::egui::FontId::monospace(*font_size as f32),
                    if theme.dark_mode {
                        eframe::egui::Color32::LIGHT_GRAY
                    } else {
                        eframe::egui::Color32::DARK_GRAY
                    },
                    f32::INFINITY,
                )
            })
    }

    fn highlight_impl(
        &self,
        theme: &CodeTheme,
        text: &str,
        language: &str,
        font_size: &i32,
    ) -> Option<LayoutJob> {
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use eframe::egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight(line, &self.ps) {
                let fg = style.foreground;
                let text_color = eframe::egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    eframe::egui::Stroke::new(1.0, text_color)
                } else {
                    eframe::egui::Stroke::none()
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: eframe::egui::FontId::monospace(*font_size as f32),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

// ----------------------------------------------------------------------------
