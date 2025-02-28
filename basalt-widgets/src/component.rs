use ratatui::{buffer::Buffer, crossterm::event::Event, layout::Rect};

pub trait Component {
    type Model: Clone;
    type Message;

    fn handle_event(model: &Self::Model, event: &Event) -> Option<Self::Message>;

    fn update(model: &Self::Model, message: Option<Self::Message>) -> Self::Model;

    fn render(model: &mut Self::Model, area: Rect, buf: &mut Buffer);
}
