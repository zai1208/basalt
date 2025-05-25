//! # Markdown View Widget
//!
//! This module provides a widget called `MarkdownView` that can render Markdown content into
//! terminal user interface (TUI) structures using the [`ratatui`](https://docs.rs/ratatui) crate.
//! It integrates with a [`super::state::MarkdownViewState`] to manage scrolling and additional
//! metadata.
//!
//! The module uses markdown parser [`basalt_core::markdown`] to produce
//! [`basalt_core::markdown::Node`] values. Each node is converted to one or more
//! [`ratatui::text::Line`] objects.
//!
//! # Example of rendered output
//!
//! Headings
//! ════════════════════════════════════════════════════════════════
//!
//! THIS IS A HEADING 1
//! ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
//!
//! This is a heading 2
//! ════════════════════════════════════════════════════════════════
//! ⬤  This is a heading 3
//!
//! ● This is a heading 4
//!
//! ◆ 𝓣𝓱𝓲𝓼 𝓲𝓼 𝓪 𝓱𝓮𝓪𝓭𝓲𝓷𝓰 𝟓
//!
//! ✺ 𝓣𝓱𝓲𝓼 𝓲𝓼 𝓪 𝓱𝓮𝓪𝓭𝓲𝓷𝓰 𝟔
//!
//! Quotes
//! ════════════════════════════════════════════════════════════════
//! You can quote text by adding a > symbols before the text.
//!
//! ┃ Human beings face ever more complex and urgent problems, and
//! ┃ their effectiveness in dealing with these problems is a matter
//! ┃ that is critical to the stability and continued progress of
//! ┃ society.
//! ┃
//! ┃ - Doug Engelbart, 1961
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Stylize},
    text::{Line, Span},
    widgets::{
        self, Block, BorderType, Padding, Paragraph, ScrollbarOrientation, StatefulWidget,
        StatefulWidgetRef, Widget,
    },
};

use crate::stylized_text::{stylize, FontStyle};

use super::parser;

use super::state::MarkdownViewState;

/// A widget for rendering markdown text using [`MarkdownViewState`].
///
/// # Example
///
/// ```rust
/// use basalt_core::markdown;
/// use basalt_widgets::markdown::{MarkdownViewState, MarkdownView};
/// use ratatui::prelude::*;
/// use ratatui::widgets::StatefulWidgetRef;
///
/// let text = "# Hello, world!\nThis is a test.";
/// let mut state = MarkdownViewState::new(text);
///
/// let area = Rect::new(0, 0, 20, 10);
/// let mut buffer = Buffer::empty(area);
///
/// MarkdownView.render_ref(area, &mut buffer, &mut state);
///
/// let expected = [
///   "╭──────────────────▲",
///   "│                  █",
///   "│ HELLO, WORLD!    █",
///   "│ ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀ █",
///   "│ This is a test.  █",
///   "│                  █",
///   "│                  █",
///   "│                  █",
///   "│                  ║",
///   "╰──────────────────▼",
/// ];
///
/// // FIXME: Take styles into account
/// // assert_eq!(buffer, Buffer::with_lines(expected));
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MarkdownView;

impl MarkdownView {
    fn task<'a>(
        kind: parser::TaskListItemKind,
        content: Vec<Span<'a>>,
        prefix: Span<'a>,
    ) -> Line<'a> {
        match kind {
            parser::TaskListItemKind::Unchecked => Line::from(
                [prefix, "□ ".dark_gray()]
                    .into_iter()
                    .chain(content)
                    .collect::<Vec<_>>(),
            ),
            parser::TaskListItemKind::Checked => Line::from(
                [prefix, "■ ".magenta()]
                    .into_iter()
                    .chain(content)
                    .collect::<Vec<_>>(),
            )
            .dark_gray()
            .add_modifier(Modifier::CROSSED_OUT),
            parser::TaskListItemKind::LooselyChecked => Line::from(
                [prefix, "■ ".magenta()]
                    .into_iter()
                    .chain(content)
                    .collect::<Vec<_>>(),
            ),
        }
    }

    fn item<'a>(kind: parser::ItemKind, content: Vec<Span<'a>>, prefix: Span<'a>) -> Line<'a> {
        match kind {
            parser::ItemKind::Ordered(num) => Line::from(
                [prefix, num.to_string().dark_gray(), ". ".into()]
                    .into_iter()
                    .chain(content)
                    .collect::<Vec<_>>(),
            ),
            parser::ItemKind::Unordered => Line::from(
                [prefix, "- ".dark_gray()]
                    .into_iter()
                    .chain(content)
                    .collect::<Vec<_>>(),
            ),
        }
    }

    fn text_to_spans<'a>(text: parser::Text) -> Vec<Span<'a>> {
        text.into_iter()
            .map(|text| Span::from(text.content))
            .collect()
    }

    fn code_block<'a>(text: parser::Text, width: usize) -> Vec<Line<'a>> {
        text.into_iter()
            .flat_map(|text| {
                text.content
                    .clone()
                    .split("\n")
                    .map(|line| {
                        format!(
                            " {} {}",
                            line,
                            // We subtract two to take the white space into account, which are
                            // added in the format string.
                            (line.chars().count()..width - 2)
                                .map(|_| " ")
                                .collect::<String>()
                        )
                    })
                    .collect::<Vec<String>>()
            })
            .map(|text| Line::from(text).bg(Color::Black))
            .collect()
    }

    fn wrap_with_prefix(text: String, width: usize, prefix: Span) -> Vec<Line> {
        let options =
            textwrap::Options::new(width.saturating_sub(prefix.width())).break_words(false);

        textwrap::wrap(&text, &options)
            .into_iter()
            .map(|wrapped_line| {
                Line::from([prefix.clone(), Span::from(wrapped_line.to_string())].to_vec())
            })
            .collect()
    }

    fn heading<'a>(level: parser::HeadingLevel, text: String, width: usize) -> Vec<Line<'a>> {
        match level {
            parser::HeadingLevel::H1 => [
                Line::default(),
                Line::from(text.to_uppercase()).italic().bold(),
                (0..width).map(|_| "▀").collect::<String>().into(),
                Line::default(),
            ]
            .to_vec(),
            parser::HeadingLevel::H2 => [
                Line::from(text).bold().yellow(),
                Line::from((0..width).map(|_| "═").collect::<String>()).yellow(),
            ]
            .to_vec(),
            parser::HeadingLevel::H3 => [
                Line::from(["⬤  ".into(), text.bold()].to_vec()).cyan(),
                Line::default(),
            ]
            .to_vec(),
            parser::HeadingLevel::H4 => [
                Line::from(["● ".into(), text.bold()].to_vec()).magenta(),
                Line::default(),
            ]
            .to_vec(),
            parser::HeadingLevel::H5 => [
                Line::from(["◆ ".into(), stylize(&text, FontStyle::Script).into()].to_vec()),
                Line::default(),
            ]
            .to_vec(),
            parser::HeadingLevel::H6 => [
                Line::from(["✺ ".into(), stylize(&text, FontStyle::Script).into()].to_vec()),
                Line::default(),
            ]
            .to_vec(),
        }
    }

    fn render_markdown<'a>(node: parser::Node, area: Rect, prefix: Span<'a>) -> Vec<Line<'a>> {
        match node.markdown_node {
            parser::MarkdownNode::Paragraph { text } => {
                MarkdownView::wrap_with_prefix(text.into(), area.width.into(), prefix.clone())
                    .into_iter()
                    .chain(if prefix.to_string().is_empty() {
                        [Line::default()].to_vec()
                    } else {
                        [].to_vec()
                    })
                    .collect::<Vec<_>>()
            }
            parser::MarkdownNode::Heading { level, text } => {
                MarkdownView::heading(level, text.into(), area.width.into())
            }
            parser::MarkdownNode::Item { text } => [MarkdownView::item(
                parser::ItemKind::Unordered,
                MarkdownView::text_to_spans(text),
                prefix,
            )]
            .to_vec(),
            parser::MarkdownNode::TaskListItem { kind, text } => [MarkdownView::task(
                kind,
                MarkdownView::text_to_spans(text),
                prefix,
            )]
            .to_vec(),
            // TODO: Add lang support and syntax highlighting
            parser::MarkdownNode::CodeBlock { text, .. } => {
                [Line::from((0..area.width).map(|_| " ").collect::<String>()).bg(Color::Black)]
                    .into_iter()
                    .chain(MarkdownView::code_block(text, area.width.into()))
                    .chain([Line::default()])
                    .collect::<Vec<_>>()
            }
            parser::MarkdownNode::List { nodes, kind } => nodes
                .into_iter()
                .enumerate()
                .flat_map(|(i, child)| match child.markdown_node {
                    parser::MarkdownNode::TaskListItem { kind, text } => [MarkdownView::task(
                        kind,
                        MarkdownView::text_to_spans(text),
                        prefix.clone(),
                    )]
                    .to_vec(),
                    parser::MarkdownNode::Item { text } => {
                        let item = match kind {
                            parser::ListKind::Ordered(start) => MarkdownView::item(
                                parser::ItemKind::Ordered(start + i as u64),
                                MarkdownView::text_to_spans(text),
                                prefix.clone(),
                            ),
                            _ => MarkdownView::item(
                                parser::ItemKind::Unordered,
                                MarkdownView::text_to_spans(text),
                                prefix.clone(),
                            ),
                        };

                        [item].to_vec()
                    }
                    _ => MarkdownView::render_markdown(
                        child,
                        area,
                        Span::from(format!("  {}", prefix)),
                    ),
                })
                .chain(if prefix.to_string().is_empty() {
                    [Line::default()].to_vec()
                } else {
                    [].to_vec()
                })
                .collect::<Vec<Line<'a>>>(),

            // TODO: Support callout block quote types
            parser::MarkdownNode::BlockQuote { nodes, .. } => nodes
                .iter()
                .map(|child| {
                    // We need this to be a block of lines to make sure we enumarate and add
                    // prefixed line breaks correctly.
                    [MarkdownView::render_markdown(
                        child.clone(),
                        area,
                        Span::from(prefix.to_string() + "┃ ").magenta(),
                    )]
                    .to_vec()
                })
                .enumerate()
                .flat_map(|(i, mut line_blocks)| {
                    if i != 0 && i != nodes.len() {
                        line_blocks.insert(
                            0,
                            [Line::from(prefix.to_string() + "┃ ").magenta()].to_vec(),
                        );
                    }
                    line_blocks.into_iter().flatten().collect::<Vec<_>>()
                })
                .chain(if prefix.to_string().is_empty() {
                    [Line::default()].to_vec()
                } else {
                    [].to_vec()
                })
                .collect::<Vec<Line<'a>>>(),
        }
    }
}

impl StatefulWidgetRef for MarkdownView {
    type State = MarkdownViewState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .padding(Padding::horizontal(1));

        let nodes = parser::from_str(&state.text)
            .into_iter()
            .flat_map(|node| {
                MarkdownView::render_markdown(node, block.inner(area), Span::default())
            })
            .collect::<Vec<Line<'_>>>();

        let mut scroll_state = state.scrollbar.state.content_length(nodes.len());

        let root_node = Paragraph::new(nodes)
            .block(block)
            .scroll((state.scrollbar.position as u16, 0));

        Widget::render(root_node, area, buf);

        StatefulWidget::render(
            widgets::Scrollbar::new(ScrollbarOrientation::VerticalRight),
            area,
            buf,
            &mut scroll_state,
        );
    }
}

// TODO: Add tests
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use indoc::indoc;
//     use ratatui::{backend::TestBackend, Terminal};
//
//     #[test]
//     fn test() {
//         let tests = [(
//             indoc! {r#"# Heading 1
//
//                 ## Heading 2
//
//                 ### Heading 3
//
//                 #### Heading 4
//
//                 ##### Heading 5
//
//                 ###### Heading 6
//                 "#},
//             indoc! {r#"
//
//                 "#},
//         )];
//
//         tests.iter().for_each(|test| {
//             let mut state = MarkdownViewState::new(test.0);
//
//             let area = Rect::new(0, 0, 20, 10);
//             let mut buffer = Buffer::empty(area);
//
//             MarkdownView.render_ref(area, &mut buffer, &mut state);
//             // println!("{:?}", terminal.backend().buffer());
//             let symbols = buffer
//                 .content()
//                 .iter()
//                 .map(|cell| cell.symbol())
//                 .collect::<Vec<&str>>();
//             assert_eq!(symbols, test.1.lines().collect::<Vec<_>>());
//         });
//     }
// }
