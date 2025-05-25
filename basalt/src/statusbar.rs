use std::marker::PhantomData;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{StatefulWidgetRef, Widget},
};

#[derive(Default, Clone, PartialEq)]
pub struct StatusBarState<'a> {
    mode: &'a str,
    meta: Option<&'a str>,
    word_count: usize,
    char_count: usize,
}

impl<'a> StatusBarState<'a> {
    pub fn new(mode: &'a str, meta: Option<&'a str>, word_count: usize, char_count: usize) -> Self {
        Self {
            mode,
            meta,
            word_count,
            char_count,
        }
    }
}

#[derive(Default)]
pub struct StatusBar<'a> {
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> StatefulWidgetRef for StatusBar<'a> {
    type State = StatusBarState<'a>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [left, right] = Layout::horizontal([Constraint::Fill(1), Constraint::Length(28)])
            .flex(Flex::SpaceBetween)
            .areas(area);

        let meta = state
            .meta
            .map(|meta| {
                [
                    Span::from(" ").bg(Color::DarkGray),
                    Span::from(meta).bg(Color::DarkGray).gray().bold(),
                    Span::from(" ").bg(Color::DarkGray),
                    Span::from("").dark_gray(),
                ]
            })
            .unwrap_or_default();

        Text::from(Line::from(
            [
                Span::from("").magenta(),
                Span::from(" ").bg(Color::Magenta),
                Span::from(state.mode).magenta().reversed().bold(),
                Span::from(" ").bg(Color::Magenta),
                Span::from("")
                    .bg(if state.meta.is_some() {
                        Color::DarkGray
                    } else {
                        Color::default()
                    })
                    .magenta(),
            ]
            .into_iter()
            .chain(meta)
            .collect::<Vec<Span>>(),
        ))
        .render(left, buf);

        let [word_count, char_count] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
                .flex(Flex::End)
                .areas(right);

        Text::from(format!("{} words", state.word_count))
            .right_aligned()
            .render(word_count, buf);

        Text::from(format!("{} chars", state.char_count))
            .right_aligned()
            .render(char_count, buf);
    }
}
