use egui::{Color32, FontFamily, FontId, RichText, Stroke, Ui};

use crate::markdown::{parse, Block, Inline};

const HEADING_SIZES: [f32; 6] = [26.0, 22.0, 18.0, 15.5, 14.0, 13.0];
const BODY_SIZE: f32 = 14.0;
const MONO_SIZE: f32 = 13.0;

pub fn show(ui: &mut Ui, source: &str, accent: Color32) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let blocks = parse(source);
            ui.add_space(4.0);
            for block in &blocks {
                render_block(ui, block, accent);
                ui.add_space(4.0);
            }
            ui.add_space(8.0);
        });
}

fn render_block(ui: &mut Ui, block: &Block, accent: Color32) {
    match block {
        Block::Heading(level, inlines) => {
            let size = HEADING_SIZES
                .get((*level as usize).saturating_sub(1))
                .copied()
                .unwrap_or(BODY_SIZE);
            ui.add_space(if *level <= 2 { 8.0 } else { 4.0 });
            ui.horizontal_wrapped(|ui| {
                for inline in inlines {
                    render_inline(ui, inline, size, accent, true);
                }
            });
            if *level <= 2 {
                ui.separator();
            }
        }
        Block::Paragraph(inlines) => {
            ui.horizontal_wrapped(|ui| {
                for inline in inlines {
                    render_inline(ui, inline, BODY_SIZE, accent, false);
                }
            });
        }
        Block::CodeBlock { code, .. } => {
            // Strip the trailing newline pulldown-cmark leaves on fenced blocks.
            let trimmed = code.trim_end_matches('\n');
            egui::Frame::group(ui.style())
                .fill(ui.visuals().extreme_bg_color)
                .stroke(Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(trimmed)
                            .font(FontId::new(MONO_SIZE, FontFamily::Monospace)),
                    );
                });
        }
        Block::BulletItem { indent, inlines } => {
            ui.horizontal_wrapped(|ui| {
                ui.add_space((*indent as f32) * 16.0);
                ui.label(RichText::new("•").monospace());
                ui.add_space(4.0);
                for inline in inlines {
                    render_inline(ui, inline, BODY_SIZE, accent, false);
                }
            });
        }
        Block::NumberedItem {
            indent,
            number,
            inlines,
        } => {
            ui.horizontal_wrapped(|ui| {
                ui.add_space((*indent as f32) * 16.0);
                ui.label(RichText::new(format!("{number}.")).monospace().weak());
                ui.add_space(4.0);
                for inline in inlines {
                    render_inline(ui, inline, BODY_SIZE, accent, false);
                }
            });
        }
        Block::Rule => {
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
        }
        Block::Quote(inlines) => {
            egui::Frame::default()
                .stroke(Stroke::new(2.0, ui.visuals().weak_text_color()))
                .inner_margin(egui::Margin {
                    left: 10.0,
                    right: 4.0,
                    top: 2.0,
                    bottom: 2.0,
                })
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for inline in inlines {
                            render_inline(ui, inline, BODY_SIZE, accent, false);
                        }
                    });
                });
        }
    }
}

fn render_inline(ui: &mut Ui, inline: &Inline, size: f32, accent: Color32, strong: bool) {
    match inline {
        Inline::Text(t) => {
            let mut text = RichText::new(t).size(size);
            if strong {
                text = text.strong();
            }
            ui.label(text);
        }
        Inline::Code(t) => {
            ui.label(
                RichText::new(t)
                    .font(FontId::new(size - 1.0, FontFamily::Monospace))
                    .background_color(ui.visuals().extreme_bg_color),
            );
        }
        Inline::Strong(t) => {
            ui.label(RichText::new(t).size(size).strong());
        }
        Inline::Emphasis(t) => {
            ui.label(RichText::new(t).size(size).italics());
        }
        Inline::Link { text, url } => {
            let resp = ui.add(
                egui::Label::new(
                    RichText::new(text)
                        .size(size)
                        .color(accent)
                        .underline(),
                )
                .sense(egui::Sense::click()),
            );
            if resp.clicked() {
                // Best-effort; failures end up in the log but don't panic.
                if let Err(err) = open_url(url) {
                    tracing::warn!(error = %err, url = %url, "open url failed");
                }
            }
            resp.on_hover_text(url);
        }
        Inline::SoftBreak => {
            ui.label(" ");
        }
        Inline::HardBreak => {
            ui.end_row();
        }
    }
}

fn open_url(url: &str) -> std::io::Result<()> {
    // macOS `open` covers https URLs cleanly; on other platforms users will
    // see the path in the log and can copy it.
    std::process::Command::new("open").arg(url).status()?;
    Ok(())
}
