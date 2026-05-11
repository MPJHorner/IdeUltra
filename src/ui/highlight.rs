use std::sync::Arc;

use egui::text::LayoutJob;
use egui::{Color32, FontFamily, FontId, TextFormat};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style};
use syntect::parsing::SyntaxReference;
use syntect::util::LinesWithEndings;

use crate::editor::language::{ColorTheme, SYNTAX_SET, THEME_SET};

/// Per-tab highlight cache. We rebuild the LayoutJob only when content,
/// theme, or wrap width changes — so most frames pay zero highlight cost.
#[derive(Default)]
pub struct HighlightCache {
    key: Option<CacheKey>,
    job: Option<Arc<LayoutJob>>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct CacheKey {
    content_hash: u64,
    syntax_name: String,
    theme: ColorTheme,
    wrap_width_bits: u32,
    font_size_bits: u32,
}

impl HighlightCache {
    pub fn layout(
        &mut self,
        text: &str,
        syntax: &SyntaxReference,
        theme: ColorTheme,
        wrap_width: f32,
        font_size: f32,
    ) -> Arc<LayoutJob> {
        let content_hash = fast_hash(text);
        let key = CacheKey {
            content_hash,
            syntax_name: syntax.name.clone(),
            theme,
            wrap_width_bits: wrap_width.to_bits(),
            font_size_bits: font_size.to_bits(),
        };
        if let Some(existing) = &self.key {
            if existing == &key {
                if let Some(job) = &self.job {
                    return job.clone();
                }
            }
        }
        let job = Arc::new(build_job(text, syntax, theme, wrap_width, font_size));
        self.key = Some(key);
        self.job = Some(job.clone());
        job
    }
}

fn build_job(
    text: &str,
    syntax: &SyntaxReference,
    theme: ColorTheme,
    wrap_width: f32,
    font_size: f32,
) -> LayoutJob {
    let theme_obj = THEME_SET
        .themes
        .get(theme.syntect_name())
        // Fallback to the first theme if the named one is missing — keeps
        // the editor rendering rather than panicking on a typo.
        .or_else(|| THEME_SET.themes.values().next())
        .expect("at least one theme is always bundled");

    let mut h = HighlightLines::new(syntax, theme_obj);
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let font_id = FontId::new(font_size, FontFamily::Monospace);

    for line in LinesWithEndings::from(text) {
        match h.highlight_line(line, &SYNTAX_SET) {
            Ok(ranges) => {
                for (style, slice) in ranges {
                    job.append(
                        slice,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: to_color(style),
                            italics: style.font_style.contains(FontStyle::ITALIC),
                            underline: if style.font_style.contains(FontStyle::UNDERLINE) {
                                egui::Stroke::new(1.0, to_color(style))
                            } else {
                                egui::Stroke::NONE
                            },
                            ..Default::default()
                        },
                    );
                }
            }
            Err(_) => {
                // On a parse error, render the line as plain monospace text
                // rather than dropping it. Better degradation than a panic.
                job.append(
                    line,
                    0.0,
                    TextFormat {
                        font_id: font_id.clone(),
                        ..Default::default()
                    },
                );
            }
        }
    }
    job
}

fn to_color(style: Style) -> Color32 {
    Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

fn fast_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
