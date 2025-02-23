//! A Markdown parser that transforms Markdown input into a custom abstract syntax tree (AST)
//! intented to be rendered with [basalt](https://github.com/erikjuhani/basalt)—a TUI application
//! for Obsidian.
//!
//! This module provides a [`Parser`] type, which processes raw Markdown input into a [`Vec`] of
//! [`Node`]s. These [`Node`]s represent semantic elements such as headings, paragraphs, block
//! quotes, and code blocks.
//!
//! The parser is built on top of [`pulldown_cmark`].
//!
//! ## Simple usage
//!
//! At the simplest level, you can parse a Markdown string by calling the [`from_str`] function:
//!
//! ```
//! use basalt_core::markdown::{from_str, Node, HeadingLevel, Text};
//!
//! let markdown = "# My Heading\n\nSome text.";
//! let nodes = from_str(markdown);
//!
//! assert_eq!(nodes, vec![
//!   Node::Heading {
//!     level: HeadingLevel::H1,
//!     text: Text::from("My Heading"),
//!   },
//!   Node::Paragraph {
//!     text: Text::from("Some text."),
//!   },
//! ])
//! ```
//!
//! ## Implementation details
//!
//! The [`Parser`] processes [`pulldown_cmark::Event`]s one by one, building up the current
//! [`Node`] in `current_node`. When an event indicates the start of a new structure (e.g.,
//! `Event::Start(Tag::Heading {..})`), the [`Parser`] pushes or replaces the current node
//! with a new one. When an event indicates the end of that structure, the node is finalized
//! and pushed into [`Parser::output`].
//!
//! Unrecognized events (such as [`InlineHtml`](pulldown_cmark::Event::InlineHtml)) are simply
//! ignored for the time being.
//!
//! ## Not yet implemented
//!
//! - Handling of inline HTML, math blocks, etc.
//! - Tracking code block language (`lang`) properly (currently set to [`None`]).
use std::vec::IntoIter;

use pulldown_cmark::{Event, Options, Tag, TagEnd};

/// A style that can be applied to [`TextNode`] (code, emphasis, strikethrough, strong).
#[derive(Clone, Debug, PartialEq)]
pub enum Style {
    /// Inline code style (e.g. `code`).
    Code,
    /// Italic/emphasis style (e.g. `*emphasis*`).
    Emphasis,
    /// Strikethrough style (e.g. `~~strikethrough~~`).
    Strikethrough,
    /// Bold/strong style (e.g. `**strong**`).
    Strong,
}

/// Represents the variant of a list or task item (checked, unchecked, etc.).
#[derive(Clone, Debug, PartialEq)]
pub enum ItemKind {
    /// A checkbox item that is marked as done using `- [x]`.
    HardChecked,
    /// A checkbox item that is checked, but not explicitly recognized as
    /// `HardChecked` (e.g., `- [?]`).
    Checked,
    /// A checkbox item that is unchecked using `- [ ]`.
    Unchecked,
    // TODO: Remove in favor of using List node that has children of nodes
    /// An ordered list item (e.g., `1. item`), storing the numeric index.
    Ordered(u64),
    /// An unordered list item (e.g., `- item`).
    Unordered,
}

#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum HeadingLevel {
    H1 = 1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<pulldown_cmark::HeadingLevel> for HeadingLevel {
    fn from(value: pulldown_cmark::HeadingLevel) -> Self {
        match value {
            pulldown_cmark::HeadingLevel::H1 => HeadingLevel::H1,
            pulldown_cmark::HeadingLevel::H2 => HeadingLevel::H2,
            pulldown_cmark::HeadingLevel::H3 => HeadingLevel::H3,
            pulldown_cmark::HeadingLevel::H4 => HeadingLevel::H4,
            pulldown_cmark::HeadingLevel::H5 => HeadingLevel::H5,
            pulldown_cmark::HeadingLevel::H6 => HeadingLevel::H6,
        }
    }
}

/// Represents specialized block quote kind variants (tip, note, warning, etc.).
///
/// Currently, the underlying [`pulldown_cmark`] parser distinguishes these via syntax like `">
/// [!NOTE] Some note"`.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum BlockQuoteKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl From<pulldown_cmark::BlockQuoteKind> for BlockQuoteKind {
    fn from(value: pulldown_cmark::BlockQuoteKind) -> Self {
        match value {
            pulldown_cmark::BlockQuoteKind::Tip => BlockQuoteKind::Tip,
            pulldown_cmark::BlockQuoteKind::Note => BlockQuoteKind::Note,
            pulldown_cmark::BlockQuoteKind::Warning => BlockQuoteKind::Warning,
            pulldown_cmark::BlockQuoteKind::Caution => BlockQuoteKind::Caution,
            pulldown_cmark::BlockQuoteKind::Important => BlockQuoteKind::Important,
        }
    }
}

/// Denotes whether a list is ordered or unordered.
#[derive(Clone, Debug, PartialEq)]
pub enum ListKind {
    /// An ordered list item (e.g., `1. item`), storing the numeric index.
    Ordered(u64),
    /// An unordered list item (e.g., `- item`).
    Unordered,
}

/// A single unit of text that is optionally styled (e.g., code).
///
/// [`TextNode`] can be any combination of sentence, words or characters.
///
/// Usually styled text will be contained in a single [`TextNode`] with the given [`Style`]
/// property.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextNode {
    /// The literal text content.
    pub content: String,
    /// Optional inline style of the text.
    pub style: Option<Style>,
}

impl TextNode {
    /// Creates a new [`TextNode`] from `content` and optional [`Style`].
    pub fn new(content: String, style: Option<Style>) -> Self {
        Self { content, style }
    }
}

/// A wrapper type holding a list of [`TextNode`]s.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Text(Vec<TextNode>);

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self(vec![TextNode::new(String::from(value), None)])
    }
}

impl IntoIterator for Text {
    type Item = TextNode;
    type IntoIter = IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Text {
    /// Appends a [`TextNode`] to the inner text list.
    fn push(&mut self, node: TextNode) {
        self.0.push(node);
    }
}

/// The Markdown AST node enumeration.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum Node {
    /// A heading node that represents different heading levels.
    ///
    /// The level is controlled with the [`HeadingLevel`] definition.
    Heading {
        level: HeadingLevel,
        text: Text,
    },
    Paragraph {
        text: Text,
    },
    /// A block quote node that represents different quote block variants including callout blocks.
    ///
    /// The variant is controlled with the [`BlockQuoteKind`] definition. When [`BlockQuoteKind`]
    /// is [`None`] the block quote should be interpreted as a regular block quote:
    /// `"> Block quote"`.
    BlockQuote {
        kind: Option<BlockQuoteKind>,
        nodes: Vec<Node>,
    },
    /// A fenced code block, optionally with a language identifier.
    CodeBlock {
        lang: Option<String>,
        text: Text,
    },
    /// A list item node that represents different list item variants including task items.
    ///
    /// The variant is controlled with the [`ItemKind`] definition. When [`ItemKind`] is [`None`]
    /// the item should be interpreted as unordered list item: `"- Item"`.
    Item {
        kind: Option<ItemKind>,
        text: Text,
    },
}

impl Node {
    /// Pushes a [`TextNode`] into this node, if it contains a text buffer.
    ///
    /// If the node is a [`BlockQuote`], the [`TextNode`] will be pushed into the last child
    /// [`Node`], if any.
    /// ```
    pub(crate) fn push_text_node(&mut self, node: TextNode) {
        match self {
            Node::Paragraph { text, .. }
            | Node::Heading { text, .. }
            | Node::CodeBlock { text, .. }
            | Node::Item { text, .. } => text.push(node),
            Node::BlockQuote { nodes, .. } => {
                if let Some(last_node) = nodes.last_mut() {
                    last_node.push_text_node(node);
                }
            }
        }
    }
}

/// Returns `true` if the [`Node`] should be closed upon encountering the given [`TagEnd`].
fn matches_tag_end(node: &Node, tag_end: &TagEnd) -> bool {
    match (node, tag_end) {
        (Node::Paragraph { .. }, TagEnd::Paragraph)
        | (Node::Heading { .. }, TagEnd::Heading(..))
        | (Node::BlockQuote { .. }, TagEnd::BlockQuote(..))
        | (Node::CodeBlock { .. }, TagEnd::CodeBlock)
        | (Node::Item { .. }, TagEnd::Item) => true,
        _ => false,
    }
}

/// Parses the given Markdown input into a list of [`Node`]s.
///
/// This is a convenience function for constructing a [`Parser`] and calling [`Parser::parse`].  
///
/// # Examples
///
/// ```
/// use basalt_core::markdown::{from_str, Node, HeadingLevel, Text};
///
/// let markdown = "# My Heading\n\nSome text.";
/// let nodes = from_str(markdown);
///
/// assert_eq!(nodes, vec![
///   Node::Heading {
///     level: HeadingLevel::H1,
///     text: Text::from("My Heading"),
///   },
///   Node::Paragraph {
///     text: Text::from("Some text."),
///   },
/// ])
/// ```
pub fn from_str<'a>(text: &'a str) -> Vec<Node> {
    Parser::new(text).parse()
}

/// A parser that consumes [`pulldown_cmark::Event`]s and produces a [`Vec`] of [`Node`].
///
/// # Examples
///
/// ```
/// use basalt_core::markdown::{Parser, Node, HeadingLevel, Text};
///
/// let markdown = "# My Heading\n\nSome text.";
/// let parser = Parser::new(markdown);
/// let nodes = parser.parse();
///
/// assert_eq!(nodes, vec![
///   Node::Heading {
///     level: HeadingLevel::H1,
///     text: Text::from("My Heading"),
///   },
///   Node::Paragraph {
///     text: Text::from("Some text."),
///   },
/// ])
/// ```
pub struct Parser<'a> {
    /// Contains the completed AST [`Node`]s.
    pub output: Vec<Node>,
    inner: pulldown_cmark::TextMergeStream<'a, pulldown_cmark::Parser<'a>>,
    current_node: Option<Node>,
}

impl<'a> Iterator for Parser<'a> {
    type Item = Event<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a> Parser<'a> {
    /// Creates a new [`Parser`] from a Markdown input string.
    ///
    /// The parser uses [`pulldown_cmark::Parser::new_ext`] with [`Options::all()`] and
    /// [`pulldown_cmark::TextMergeStream`] internally.
    pub fn new(text: &'a str) -> Self {
        let parser = pulldown_cmark::TextMergeStream::new(pulldown_cmark::Parser::new_ext(
            text,
            Options::all(),
        ));

        Self {
            inner: parser,
            output: vec![],
            current_node: None,
        }
    }

    /// Pushes a [`Node`] as a child if the current node is a [`BlockQuote`], otherwise sets it as
    /// the `current_node`.
    fn push_node(&mut self, node: Node) {
        if let Some(Node::BlockQuote { nodes, .. }) = &mut self.current_node {
            nodes.push(node);
        } else {
            self.set_node(&node);
        }
    }

    /// Pushes a [`TextNode`] into the `current_node` if it exists.
    fn push_text_node(&mut self, node: TextNode) {
        if let Some(ref mut current) = self.current_node {
            current.push_text_node(node);
        }
    }

    /// Sets (or replaces) the `current_node` with a new one, discarding any old node.
    fn set_node(&mut self, block: &Node) {
        self.current_node.replace(block.clone());
    }

    /// Handles the start of a [`Tag`]. Pushes the matching semantic node to be processed.
    fn tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::Paragraph => self.push_node(Node::Paragraph {
                text: Text::default(),
            }),
            Tag::Heading { level, .. } => self.push_node(Node::Heading {
                level: level.into(),
                text: Text::default(),
            }),
            Tag::BlockQuote(kind) => self.push_node(Node::BlockQuote {
                kind: kind.map(|kind| kind.into()),
                nodes: vec![],
            }),
            Tag::CodeBlock(_) => self.push_node(Node::CodeBlock {
                lang: None,
                text: Text::default(),
            }),
            Tag::Item => self.push_node(Node::Item {
                kind: None,
                text: Text::default(),
            }),
            // For now everything below this comment are defined as paragraph nodes
            Tag::HtmlBlock
            | Tag::List(_)
            | Tag::FootnoteDefinition(_)
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Link { .. }
            | Tag::Image { .. }
            | Tag::MetadataBlock(_)
            | Tag::DefinitionList
            | Tag::DefinitionListTitle
            | Tag::DefinitionListDefinition => self.push_node(Node::Paragraph {
                text: Text::default(),
            }),
        }
    }

    /// Handles the end of a [`Tag`], finalizing a node if matching.
    fn tag_end(&mut self, tag_end: TagEnd) {
        let Some(node) = self.current_node.take() else {
            return;
        };

        if matches_tag_end(&node, &tag_end) {
            self.output.push(node);
        } else {
            self.set_node(&node);
        }
    }

    /// Processes a single [`Event`] from the underlying [`pulldown_cmark::Parser`] iterator.
    fn handle_event(&mut self, event: Event<'a>) {
        match event {
            Event::Start(tag) => self.tag(tag),
            Event::End(tag_end) => self.tag_end(tag_end),
            Event::Text(text) => self.push_text_node(TextNode::new(text.to_string(), None)),
            Event::Code(text) => {
                self.push_text_node(TextNode::new(text.to_string(), Some(Style::Code)))
            }
            Event::TaskListMarker(checked) => {
                if checked {
                    self.set_node(&Node::Item {
                        kind: Some(ItemKind::HardChecked),
                        text: Text::default(),
                    });
                } else {
                    self.set_node(&Node::Item {
                        kind: Some(ItemKind::Unchecked),
                        text: Text::default(),
                    });
                }
            }
            Event::InlineMath(_)
            | Event::DisplayMath(_)
            | Event::Html(_)
            | Event::InlineHtml(_)
            | Event::SoftBreak
            | Event::HardBreak
            | Event::Rule
            | Event::FootnoteReference(_) => {
                // TODO: Not yet implemented
            }
        }
    }

    /// Consumes the parser, processing all remaining events from the stream into a list of
    /// [`Node`]s.
    ///
    /// # Examples
    ///
    /// ```
    /// # use basalt_core::markdown::{Parser, Node, Text};
    /// let parser = Parser::new("Hello world");
    ///
    /// let nodes = parser.parse();
    ///
    /// assert_eq!(nodes, vec![Node::Paragraph { text: Text::from("Hello world") }]);
    /// ```
    pub fn parse(mut self) -> Vec<Node> {
        while let Some(event) = self.next() {
            self.handle_event(event);
        }

        if let Some(node) = self.current_node.take() {
            self.output.push(node);
        }

        self.output
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    fn text(str: &str) -> Text {
        Text(vec![TextNode::new(String::from(str), None)])
    }

    fn p(str: &str) -> Node {
        Node::Paragraph { text: text(str) }
    }

    fn blockquote(nodes: Vec<Node>) -> Node {
        Node::BlockQuote { kind: None, nodes }
    }

    fn item(str: &str) -> Node {
        Node::Item {
            kind: None,
            text: text(str),
        }
    }

    fn task(str: &str) -> Node {
        Node::Item {
            kind: Some(ItemKind::Unchecked),
            text: text(str),
        }
    }

    fn completed_task(str: &str) -> Node {
        Node::Item {
            kind: Some(ItemKind::HardChecked),
            text: text(str),
        }
    }

    fn heading(level: HeadingLevel, str: &str) -> Node {
        Node::Heading {
            level,
            text: text(str),
        }
    }

    fn h1(str: &str) -> Node {
        heading(HeadingLevel::H1, str)
    }

    fn h2(str: &str) -> Node {
        heading(HeadingLevel::H2, str)
    }

    fn h3(str: &str) -> Node {
        heading(HeadingLevel::H3, str)
    }

    fn h4(str: &str) -> Node {
        heading(HeadingLevel::H4, str)
    }

    fn h5(str: &str) -> Node {
        heading(HeadingLevel::H5, str)
    }

    fn h6(str: &str) -> Node {
        heading(HeadingLevel::H6, str)
    }

    use super::*;

    #[test]
    fn test_parse() {
        let tests = [
            (
                indoc! {r#"# Heading 1

                ## Heading 2

                ### Heading 3

                #### Heading 4

                ##### Heading 5

                ###### Heading 6
                "#},
                vec![
                    h1("Heading 1"),
                    h2("Heading 2"),
                    h3("Heading 3"),
                    h4("Heading 4"),
                    h5("Heading 5"),
                    h6("Heading 6"),
                ],
            ),
            (
                indoc! {r#"Paragraph

                > BlockQuote
                >
                > - List item in BlockQuote
                "#},
                vec![
                    p("Paragraph"),
                    blockquote(vec![
                        p("BlockQuote"),
                        Node::Paragraph {
                            text: Text::default(),
                        },
                        item("List item in BlockQuote"),
                    ]),
                ],
            ),
            // TODO: Implement correct test case when `- [?] ` task item syntax is supported
            // Now we interpret it as a regular paragraph
            (
                indoc! { r#"## Tasks

                - [ ] Task

                - [x] Completed task

                - [?] Completed task
                "#},
                vec![
                    h2("Tasks"),
                    task("Task"),
                    completed_task("Completed task"),
                    p("[?] Completed task"),
                ],
            ),
        ];

        tests
            .iter()
            .for_each(|test| assert_eq!(from_str(test.0), test.1));
    }
}
