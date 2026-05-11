use std::sync::Arc;

use egui::text::{LayoutJob, LayoutSection};
use egui::{Color32, FontFamily, FontId, TextFormat};
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Style};
use syntect::parsing::SyntaxReference;
use syntect::util::LinesWithEndings;

use crate::editor::language::{ColorTheme, SYNTAX_SET, THEME_SET};

/// Per-tab highlight cache. We rebuild the LayoutJob only when content,
/// theme, wrap width, or the bracket-match positions change.
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
    /// `(open, close)` byte positions of a matched bracket pair, or
    /// `(usize::MAX, usize::MAX)` for "no match". Stored as a sentinel
    /// rather than Option<...> so the field's Hash/Eq is trivial.
    bracket_match: (usize, usize),
}

impl HighlightCache {
    pub fn layout(
        &mut self,
        text: &str,
        syntax: &SyntaxReference,
        theme: ColorTheme,
        wrap_width: f32,
        font_size: f32,
        bracket_match: Option<(usize, usize)>,
    ) -> Arc<LayoutJob> {
        let content_hash = fast_hash(text);
        let key = CacheKey {
            content_hash,
            syntax_name: syntax.name.clone(),
            theme,
            wrap_width_bits: wrap_width.to_bits(),
            font_size_bits: font_size.to_bits(),
            bracket_match: bracket_match.unwrap_or((usize::MAX, usize::MAX)),
        };
        if let Some(existing) = &self.key {
            if existing == &key {
                if let Some(job) = &self.job {
                    return job.clone();
                }
            }
        }
        let mut job = build_job(text, syntax, theme, wrap_width, font_size);
        if let Some((a, b)) = bracket_match {
            let bg = bracket_match_bg(theme);
            inject_background(&mut job, a, bg);
            inject_background(&mut job, b, bg);
        }
        let job = Arc::new(job);
        self.key = Some(key);
        self.job = Some(job.clone());
        job
    }
}

fn bracket_match_bg(theme: ColorTheme) -> Color32 {
    match theme {
        ColorTheme::Dark => Color32::from_rgba_premultiplied(80, 130, 200, 60),
        ColorTheme::Light => Color32::from_rgba_premultiplied(180, 210, 255, 200),
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

/// Tint the byte at `byte_pos` with `bg` by splitting the section that
/// contains it into up to three parts. The bracket character is always
/// a single ASCII byte, so we tint exactly one byte.
fn inject_background(job: &mut LayoutJob, byte_pos: usize, bg: Color32) {
    let idx = match job
        .sections
        .iter()
        .position(|s| s.byte_range.contains(&byte_pos))
    {
        Some(i) => i,
        None => return,
    };
    let section = job.sections[idx].clone();
    let leading = section.leading_space;
    let base = section.format.clone();

    let mut tinted = base.clone();
    tinted.background = bg;

    let bracket_end = byte_pos + 1;
    if bracket_end > section.byte_range.end {
        return;
    }

    let before = section.byte_range.start..byte_pos;
    let bracket = byte_pos..bracket_end;
    let after = bracket_end..section.byte_range.end;

    let mut replacements: Vec<LayoutSection> = Vec::with_capacity(3);
    if !before.is_empty() {
        replacements.push(LayoutSection {
            leading_space: leading,
            byte_range: before,
            format: base.clone(),
        });
    }
    replacements.push(LayoutSection {
        leading_space: if replacements.is_empty() { leading } else { 0.0 },
        byte_range: bracket,
        format: tinted,
    });
    if !after.is_empty() {
        replacements.push(LayoutSection {
            leading_space: 0.0,
            byte_range: after,
            format: base,
        });
    }

    job.sections.splice(idx..=idx, replacements);
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
