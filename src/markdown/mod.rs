//! Markdown preview support.
//!
//! `parse(text)` returns a `Vec<Block>` — a flat, unit-testable intermediate
//! representation of the document. The UI layer (`ui::markdown_preview`)
//! consumes that vector and produces egui widgets.
//!
//! We deliberately don't try to be a full CommonMark renderer. The target
//! is "looks right for a typical README" — headings, paragraphs, bold,
//! italic, inline code, fenced code blocks, bullet lists, links.

use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading(u32, Vec<Inline>),
    Paragraph(Vec<Inline>),
    CodeBlock {
        language: String,
        code: String,
    },
    BulletItem {
        indent: u32,
        inlines: Vec<Inline>,
    },
    NumberedItem {
        indent: u32,
        number: u32,
        inlines: Vec<Inline>,
    },
    Rule,
    Quote(Vec<Inline>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inline {
    Text(String),
    Code(String),
    Strong(String),
    Emphasis(String),
    Link { text: String, url: String },
    SoftBreak,
    HardBreak,
}

#[derive(Default)]
struct InlineBuilder {
    out: Vec<Inline>,
    in_strong: bool,
    in_emph: bool,
    link_url: Option<String>,
    /// Text accumulator for the active strong / emph / link span.
    span_text: String,
}

impl InlineBuilder {
    fn push_text(&mut self, t: &str) {
        if self.in_strong || self.in_emph || self.link_url.is_some() {
            self.span_text.push_str(t);
        } else if let Some(Inline::Text(last)) = self.out.last_mut() {
            last.push_str(t);
        } else {
            self.out.push(Inline::Text(t.to_string()));
        }
    }

    fn flush_span(&mut self) {
        if self.in_strong {
            self.out
                .push(Inline::Strong(std::mem::take(&mut self.span_text)));
        } else if self.in_emph {
            self.out
                .push(Inline::Emphasis(std::mem::take(&mut self.span_text)));
        } else if let Some(url) = self.link_url.take() {
            self.out.push(Inline::Link {
                text: std::mem::take(&mut self.span_text),
                url,
            });
        }
    }
}

pub fn parse(text: &str) -> Vec<Block> {
    let parser = Parser::new(text);
    let mut blocks: Vec<Block> = Vec::new();
    let mut inlines = InlineBuilder::default();
    let mut current: Option<CurrentBlock> = None;
    let mut list_stack: Vec<ListKind> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { .. } => {
                    current = Some(CurrentBlock::Heading);
                    inlines = InlineBuilder::default();
                }
                Tag::Paragraph => {
                    if current.is_none() {
                        current = Some(CurrentBlock::Paragraph);
                        inlines = InlineBuilder::default();
                    }
                }
                Tag::CodeBlock(kind) => {
                    let lang = match &kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    current = Some(CurrentBlock::CodeBlock {
                        language: lang,
                        code: String::new(),
                    });
                }
                Tag::List(start) => {
                    list_stack.push(match start {
                        Some(n) => ListKind::Numbered(n as u32),
                        None => ListKind::Bullet,
                    });
                }
                Tag::Item => {
                    let indent = (list_stack.len().saturating_sub(1)) as u32;
                    let kind = list_stack.last().copied().unwrap_or(ListKind::Bullet);
                    current = Some(CurrentBlock::Item { indent, kind });
                    inlines = InlineBuilder::default();
                    // Bump the numbered counter so the next sibling gets n+1.
                    if let Some(ListKind::Numbered(n)) = list_stack.last_mut() {
                        *n += 1;
                    }
                }
                Tag::Strong => {
                    inlines.flush_span();
                    inlines.in_strong = true;
                    inlines.span_text.clear();
                }
                Tag::Emphasis => {
                    inlines.flush_span();
                    inlines.in_emph = true;
                    inlines.span_text.clear();
                }
                Tag::Link { dest_url, .. } => {
                    inlines.flush_span();
                    inlines.link_url = Some(dest_url.to_string());
                    inlines.span_text.clear();
                }
                Tag::BlockQuote => {
                    current = Some(CurrentBlock::Quote);
                    inlines = InlineBuilder::default();
                }
                _ => {}
            },
            Event::End(end) => match end {
                TagEnd::Heading(level) => {
                    if matches!(current, Some(CurrentBlock::Heading)) {
                        inlines.flush_span();
                        blocks.push(Block::Heading(
                            level_to_u32(level),
                            std::mem::take(&mut inlines.out),
                        ));
                        current = None;
                    }
                }
                TagEnd::Paragraph => {
                    if matches!(current, Some(CurrentBlock::Paragraph)) {
                        inlines.flush_span();
                        blocks.push(Block::Paragraph(std::mem::take(&mut inlines.out)));
                        current = None;
                    }
                }
                TagEnd::CodeBlock => {
                    if let Some(CurrentBlock::CodeBlock { language, code }) =
                        current.take()
                    {
                        blocks.push(Block::CodeBlock { language, code });
                    }
                }
                TagEnd::List(_) => {
                    list_stack.pop();
                }
                TagEnd::Item => {
                    inlines.flush_span();
                    if let Some(CurrentBlock::Item { indent, kind }) = current.take() {
                        let inlines_out = std::mem::take(&mut inlines.out);
                        match kind {
                            ListKind::Bullet => blocks.push(Block::BulletItem {
                                indent,
                                inlines: inlines_out,
                            }),
                            ListKind::Numbered(n) => blocks.push(Block::NumberedItem {
                                indent,
                                // The counter was bumped before render — show n-1.
                                number: n.saturating_sub(1).max(1),
                                inlines: inlines_out,
                            }),
                        }
                    }
                }
                TagEnd::Strong => {
                    inlines.flush_span();
                    inlines.in_strong = false;
                }
                TagEnd::Emphasis => {
                    inlines.flush_span();
                    inlines.in_emph = false;
                }
                TagEnd::Link => {
                    inlines.flush_span();
                }
                TagEnd::BlockQuote => {
                    if matches!(current, Some(CurrentBlock::Quote)) {
                        inlines.flush_span();
                        blocks.push(Block::Quote(std::mem::take(&mut inlines.out)));
                        current = None;
                    }
                }
                _ => {}
            },
            Event::Text(t) => match &mut current {
                Some(CurrentBlock::CodeBlock { code, .. }) => code.push_str(&t),
                _ => inlines.push_text(&t),
            },
            Event::Code(t) => {
                if matches!(current, Some(CurrentBlock::CodeBlock { .. })) {
                    // Should never happen — Code events are inline only.
                } else {
                    inlines.flush_span();
                    inlines.out.push(Inline::Code(t.to_string()));
                }
            }
            Event::SoftBreak => inlines.out.push(Inline::SoftBreak),
            Event::HardBreak => inlines.out.push(Inline::HardBreak),
            Event::Rule => blocks.push(Block::Rule),
            _ => {}
        }
    }

    blocks
}

#[derive(Debug)]
enum CurrentBlock {
    Heading,
    Paragraph,
    CodeBlock { language: String, code: String },
    Item { indent: u32, kind: ListKind },
    Quote,
}

#[derive(Debug, Clone, Copy)]
enum ListKind {
    Bullet,
    /// Stores the *next* number to render — bumped on each Item start.
    Numbered(u32),
}

fn level_to_u32(level: HeadingLevel) -> u32 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_no_blocks() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn plain_paragraph_becomes_paragraph_block() {
        let b = parse("hello world");
        assert_eq!(b.len(), 1);
        match &b[0] {
            Block::Paragraph(inlines) => {
                assert_eq!(inlines, &vec![Inline::Text("hello world".to_string())]);
            }
            other => panic!("expected paragraph, got {other:?}"),
        }
    }

    #[test]
    fn h1_heading_parsed() {
        let b = parse("# Title");
        assert_eq!(b.len(), 1);
        match &b[0] {
            Block::Heading(level, inlines) => {
                assert_eq!(*level, 1);
                assert_eq!(inlines, &vec![Inline::Text("Title".to_string())]);
            }
            other => panic!("expected heading, got {other:?}"),
        }
    }

    #[test]
    fn h3_heading_keeps_level() {
        let b = parse("### Subsection");
        match &b[0] {
            Block::Heading(level, _) => assert_eq!(*level, 3),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn fenced_code_block_captures_language_and_body() {
        let src = "```rust\nfn main() {}\n```";
        let b = parse(src);
        assert_eq!(b.len(), 1);
        match &b[0] {
            Block::CodeBlock { language, code } => {
                assert_eq!(language, "rust");
                assert!(code.contains("fn main()"));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn inline_code_is_captured_as_code_variant() {
        let b = parse("Use `Vec::new()` to make one.");
        match &b[0] {
            Block::Paragraph(inlines) => {
                assert!(inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Code(s) if s == "Vec::new()")));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn bold_and_italic_become_distinct_inlines() {
        let b = parse("**bold** and *italic*");
        match &b[0] {
            Block::Paragraph(inlines) => {
                assert!(inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Strong(s) if s == "bold")));
                assert!(inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Emphasis(s) if s == "italic")));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn link_captures_text_and_url() {
        let b = parse("[click](https://example.com)");
        match &b[0] {
            Block::Paragraph(inlines) => {
                let link = inlines.iter().find_map(|i| match i {
                    Inline::Link { text, url } => Some((text.as_str(), url.as_str())),
                    _ => None,
                });
                assert_eq!(link, Some(("click", "https://example.com")));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn bullet_list_produces_bullet_items() {
        let b = parse("- one\n- two\n- three");
        let count = b
            .iter()
            .filter(|block| matches!(block, Block::BulletItem { .. }))
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn numbered_list_produces_numbered_items() {
        let b = parse("1. first\n2. second");
        let count = b
            .iter()
            .filter(|block| matches!(block, Block::NumberedItem { .. }))
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn horizontal_rule_becomes_rule_block() {
        let b = parse("foo\n\n---\n\nbar");
        assert!(b.iter().any(|block| matches!(block, Block::Rule)));
    }

    #[test]
    fn blockquote_becomes_quote_block() {
        let b = parse("> a quote");
        match &b[0] {
            Block::Quote(inlines) => {
                assert!(inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Text(s) if s.contains("a quote"))));
            }
            other => panic!("got {other:?}"),
        }
    }
}
