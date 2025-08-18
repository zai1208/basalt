use basalt_core::obsidian::{Note, Vault, VaultEntry};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect, Size},
    widgets::{StatefulWidget, StatefulWidgetRef},
    DefaultTerminal,
};

use std::{cell::RefCell, fmt::Debug, io::Result};

use crate::{
    config::{self, Config},
    explorer::{Explorer, ExplorerState},
    help_modal::{HelpModal, HelpModalState},
    note_editor::{Editor, EditorState, Mode},
    splash::{Splash, SplashState},
    statusbar::{StatusBar, StatusBarState},
    stylized_text::{self, FontStyle},
    text_counts::{CharCount, WordCount},
    vault_selector_modal::{VaultSelectorModal, VaultSelectorModalState},
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP_TEXT: &str = include_str!("./help.txt");

#[derive(Debug, Default, Clone, PartialEq)]
pub enum ScrollAmount {
    #[default]
    One,
    HalfPage,
}

fn calc_scroll_amount(scroll_amount: ScrollAmount, height: usize) -> usize {
    match scroll_amount {
        ScrollAmount::One => 1,
        ScrollAmount::HalfPage => height / 2,
    }
}

#[derive(Default, Clone)]
struct MainState<'a> {
    active_pane: ActivePane,
    explorer: ExplorerState<'a>,
    note_editor: EditorState<'a>,
    selected_note: Option<SelectedNote>,
}

impl<'a> MainState<'a> {
    fn new(selected_vault_name: &'a str, notes: Vec<VaultEntry>) -> Self {
        Self {
            active_pane: ActivePane::Explorer,
            explorer: ExplorerState::new(selected_vault_name, notes).set_active(true),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone)]
pub struct AppState<'a> {
    screen: ScreenState<'a>,
    screen_size: Size,
    is_running: bool,

    help_modal: HelpModalState,
    vault_selector_modal: VaultSelectorModalState<'a>,
}

fn modal_area_height(size: Size) -> usize {
    let vertical = Layout::vertical([Constraint::Percentage(50)]).flex(Flex::Center);
    let [area] = vertical.areas(Rect::new(0, 0, size.width, size.height.saturating_sub(3)));
    area.height.into()
}

#[derive(Clone)]
enum ScreenState<'a> {
    Splash(SplashState<'a>),
    Main(Box<MainState<'a>>),
}

impl<'a> AppState<'a> {
    pub fn active_component(&self) -> ActivePane {
        if self.help_modal.visible {
            return ActivePane::HelpModal;
        }

        if self.vault_selector_modal.visible {
            return ActivePane::VaultSelectorModal;
        }

        match &self.screen {
            ScreenState::Splash(..) => ActivePane::Splash,
            ScreenState::Main(state) => state.active_pane,
        }
    }

    pub fn set_running(&self, is_running: bool) -> Self {
        Self {
            is_running,
            ..self.clone()
        }
    }

    fn with_vault_selector_modal_state(
        &self,
        vault_selector_modal: VaultSelectorModalState<'a>,
    ) -> Self {
        Self {
            vault_selector_modal,
            ..self.clone()
        }
    }

    fn with_help_modal_state(&self, help_modal: HelpModalState) -> Self {
        Self {
            help_modal,
            ..self.clone()
        }
    }

    fn with_main_state(&self, main_state: MainState<'a>) -> Self {
        Self {
            screen: ScreenState::Main(Box::new(main_state)),
            ..self.clone()
        }
    }

    fn with_splash_state(&self, splash_state: SplashState<'a>) -> Self {
        Self {
            screen: ScreenState::Splash(splash_state),
            ..self.clone()
        }
    }
}

impl Default for ScreenState<'_> {
    fn default() -> Self {
        Self::Splash(SplashState::default())
    }
}

pub mod splash {
    use crate::splash::SplashState;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Message {
        Up,
        Down,
        Open,
    }

    pub fn update(message: Message, state: SplashState) -> SplashState {
        match message {
            Message::Up => state.previous(),
            Message::Down => state.next(),
            Message::Open => state.select(),
        }
    }
}

pub mod explorer {
    use crate::explorer::ExplorerState;

    use super::ScrollAmount;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Message {
        Up,
        Down,
        Open,
        Sort,
        Toggle,
        SwitchPaneNext,
        SwitchPanePrevious,
        ScrollUp(ScrollAmount),
        ScrollDown(ScrollAmount),
    }

    pub fn update(message: Message, state: ExplorerState) -> ExplorerState {
        match message {
            Message::Up => state.previous(1),
            Message::Down => state.next(1),
            Message::Sort => state.sort(),
            Message::Open => state.select(),
            Message::Toggle => state.toggle(),
            Message::SwitchPaneNext | Message::SwitchPanePrevious => {
                if state.active {
                    state.set_active(false)
                } else {
                    state.set_active(true)
                }
            }
            _ => state,
        }
    }
}

                if state.active {
                    state.set_active(false)
                } else {
                    state.set_active(true)
                }
            }
            _ => state,
        }
    }
}

pub mod note_editor {
    use ratatui::crossterm::event::{KeyCode, KeyEvent};

    use super::ScrollAmount;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Message {
        Save,
        SwitchPaneNext,
        SwitchPanePrevious,
        ToggleExplorer,
        EditMode,
        ExitMode,
        ReadMode,
        KeyEvent(KeyEvent),
        CursorUp,
        CursorLeft,
        CursorRight,
        CursorWordForward,
        CursorWordBackward,
        CursorDown,
        ScrollUp(ScrollAmount),
        ScrollDown(ScrollAmount),
        Delete,
    }

    pub fn handle_editing_event(key: &KeyEvent) -> Option<Message> {
        match key.code {
            KeyCode::Up => Some(Message::CursorUp),
            KeyCode::Down => Some(Message::CursorDown),
            KeyCode::Esc => Some(Message::ExitMode),
            KeyCode::Backspace => Some(Message::Delete),
            _ => Some(Message::KeyEvent(*key)),
        }
    }
}

pub mod help_modal {
    use crate::help_modal::HelpModalState;

    use super::ScrollAmount;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Message {
        Toggle,
        Close,
        ScrollUp(ScrollAmount),
        ScrollDown(ScrollAmount),
    }

    pub fn update(message: Message, state: HelpModalState) -> HelpModalState {
        match message {
            Message::Toggle => state.toggle_visibility(),
            Message::Close => state.hide(),
            _ => state,
        }
    }
}

pub mod vault_selector_modal {
    use crate::vault_selector_modal::VaultSelectorModalState;

    #[derive(Clone, Debug, PartialEq)]
    pub enum Message {
        Toggle,
        Up,
        Down,
        Select,
        Close,
    }

    pub fn update(message: Message, state: VaultSelectorModalState) -> VaultSelectorModalState {
        match message {
            Message::Up => state.previous(),
            Message::Down => state.next(),
            Message::Toggle => state.toggle_visibility(),
            Message::Select => state.select(),
            Message::Close => state.hide(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
    Quit,
    Resize(Size),

    Splash(splash::Message),
    Explorer(explorer::Message),
    NoteEditor(note_editor::Message),
    HelpModal(help_modal::Message),
    VaultSelectorModal(vault_selector_modal::Message),
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum ActivePane {
    #[default]
    Splash,
    Explorer,
    NoteEditor,
    HelpModal,
    VaultSelectorModal,
}

impl From<ActivePane> for &str {
    fn from(value: ActivePane) -> Self {
        match value {
            ActivePane::Splash => "Splash",
            ActivePane::Explorer => "Explorer",
            ActivePane::NoteEditor => "Note Editor",
            ActivePane::HelpModal => "Help",
            ActivePane::VaultSelectorModal => "Vault Selector",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SelectedNote {
    name: String,
    path: String,
    content: String,
}

impl From<Note> for SelectedNote {
    fn from(value: Note) -> Self {
        Self {
            name: value.name.clone(),
            path: value.path.to_string_lossy().to_string(),
            content: Note::read_to_string(&value).unwrap_or_default(),
        }
    }
}

fn help_text(version: &str) -> String {
    HELP_TEXT.replace("%version-notice", version)
}

pub struct App<'a> {
    state: AppState<'a>,
    config: Config,
    terminal: RefCell<DefaultTerminal>,
}

impl<'a> App<'a> {
    pub fn new(state: AppState<'a>, terminal: DefaultTerminal) -> Self {
        Self {
            state,
            // TODO: Surface toast if read config returns error
            config: config::load().unwrap(),
            terminal: RefCell::new(terminal),
        }
    }

    pub fn start(terminal: DefaultTerminal, vaults: Vec<&Vault>) -> Result<()> {
        let version = stylized_text::stylize(&format!("{VERSION}~beta"), FontStyle::Script);
        let size = terminal.size()?;

        let state = AppState {
            screen_size: size,
            help_modal: HelpModalState::new(&help_text(&version)),
            vault_selector_modal: VaultSelectorModalState::new(vaults.clone()),
            ..Default::default()
        }
        .with_splash_state(SplashState::new(&version, vaults));

        App::new(state, terminal).run()
    }

    fn run(&'a mut self) -> Result<()> {
        self.state.is_running = true;

        while self.state.is_running {
            self.draw(&mut self.state.clone())?;
            let event = event::read()?;
            let action = self.handle_event(&event);
            self.state = self.update(&self.state, action);
        }

        Ok(())
    }

    fn draw(&self, state: &'a mut AppState<'a>) -> Result<()> {
        let mut terminal = self.terminal.borrow_mut();

        terminal.draw(move |frame| {
            let area = frame.area();
            let buf = frame.buffer_mut();
            self.render_ref(area, buf, state);
        })?;

        Ok(())
    }

    fn handle_event(&self, event: &Event) -> Option<Message> {
        match event {
            Event::Resize(cols, rows) => Some(Message::Resize(Size::new(*cols, *rows))),
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => None,
        }
    }

    #[rustfmt::skip]
    fn handle_active_component_event(&self, key: &KeyEvent, active_component: ActivePane) -> Option<Message> {
        match active_component {
            ActivePane::Splash => self.config.splash.key_to_message(key.into()),
            ActivePane::Explorer => self.config.explorer.key_to_message(key.into()),
            ActivePane::NoteEditor => {
                match &self.state.screen {
                    ScreenState::Main(state) if state.note_editor.is_editing() => {
                        note_editor::handle_editing_event(key).map(Message::NoteEditor)
                    },
                    ScreenState::Main(_) =>
                        self.config.note_editor.key_to_message(key.into()),
                    _ => None
                }
            },
            ActivePane::HelpModal => self.config.help_modal.key_to_message(key.into()),
            ActivePane::VaultSelectorModal => self.config.vault_selector_modal.key_to_message(key.into()),
        }
    }

    fn handle_key_event(&self, key: &KeyEvent) -> Option<Message> {
        let global_message = self.config.global.key_to_message(key.into());

        let is_editing = match &self.state.screen {
            ScreenState::Main(state) => state.note_editor.is_editing(),
            _ => false,
        };

        if global_message.is_some() && !is_editing {
            return global_message;
        }

        let active_component = self.state.active_component();
        self.handle_active_component_event(key, active_component)
    }

    fn update(&self, state: &AppState<'a>, message: Option<Message>) -> AppState<'a> {
        let state = state.clone();
        let Some(message) = message else {
            return state;
        };

        let screen = state.screen.clone();

        match message {
            Message::Quit => state.set_running(false),
            Message::Resize(size) => AppState {
                screen_size: size,
                ..state
            },
            Message::HelpModal(message) => {
                let help_modal = help_modal::update(message.clone(), state.help_modal.clone());

                match message {
                    help_modal::Message::ScrollDown(scroll_amount) => {
                        state.with_help_modal_state(help_modal.scroll_down(calc_scroll_amount(
                            scroll_amount,
                            modal_area_height(state.screen_size),
                        )))
                    }
                    help_modal::Message::ScrollUp(scroll_amount) => {
                        state.with_help_modal_state(help_modal.scroll_up(calc_scroll_amount(
                            scroll_amount,
                            modal_area_height(state.screen_size),
                        )))
                    }
                    _ => state.with_help_modal_state(help_modal),
                }
            }
            Message::VaultSelectorModal(message) => {
                let ScreenState::Main(_) = screen else {
                    return state;
                };

                let vault_selector_modal = vault_selector_modal::update(
                    message.clone(),
                    state.vault_selector_modal.clone(),
                );

                match message {
                    vault_selector_modal::Message::Select => vault_selector_modal
                        .selected()
                        .and_then(|index| vault_selector_modal.clone().get_item(index))
                        .map(|vault| {
                            state
                                .with_main_state(MainState::new(&vault.name, vault.entries()))
                                .with_vault_selector_modal_state(vault_selector_modal.hide())
                        })
                        .unwrap_or(state),
                    _ => state.with_vault_selector_modal_state(vault_selector_modal),
                }
            }
            Message::Splash(message) => {
                let ScreenState::Splash(splash_state) = screen else {
                    return state;
                };

                let splash_state = splash::update(message.clone(), splash_state);

                match message {
                    splash::Message::Open => splash_state
                        .selected()
                        .and_then(|index| splash_state.clone().get_item(index))
                        .map(|vault| {
                            state.with_main_state(MainState::new(&vault.name, vault.entries()))
                        })
                        .unwrap_or(state),
                    _ => state.with_splash_state(splash_state),
                }
            }
            Message::Explorer(message) => {
                let ScreenState::Main(main_state) = screen else {
                    return state;
                };

                let explorer = explorer::update(message.clone(), main_state.explorer.clone());

                match message {
                    explorer::Message::SwitchPaneNext => state.with_main_state(MainState {
                        active_pane: ActivePane::NoteEditor,
                        note_editor: main_state.note_editor.set_active(true),
                        explorer,
                        ..*main_state
                    }),
                    explorer::Message::SwitchPanePrevious => state.with_main_state(MainState {
                        active_pane: ActivePane::Outline,
                        outline: main_state.outline.set_active(true),
                        explorer,
                        ..*main_state
                    }),
                    explorer::Message::ScrollUp(scroll_amount) => {
                        state.with_main_state(MainState {
                            explorer: explorer.previous(calc_scroll_amount(
                                scroll_amount,
                                state.screen_size.height.into(),
                            )),
                            ..*main_state
                        })
                    }
                    explorer::Message::ScrollDown(scroll_amount) => {
                        state.with_main_state(MainState {
                            explorer: explorer.next(calc_scroll_amount(
                                scroll_amount,
                                state.screen_size.height.into(),
                            )),
                            ..*main_state
                        })
                    }
                    explorer::Message::Toggle => state.with_main_state(match explorer.open {
                        true => MainState {
                            explorer,
                            ..*main_state
                        },
                        false => MainState {
                            active_pane: ActivePane::NoteEditor,
                            explorer: explorer.set_active(false),
                            note_editor: main_state.note_editor.set_active(true),
                            ..*main_state
                        },
                    }),
                    explorer::Message::Open => {
                        let selected_note = explorer.selected_note.clone().map(SelectedNote::from);
                        let note_editor = selected_note
                            .clone()
                            .map(|note| {
                                EditorState::default()
                                    .set_mode(if self.config.experimental_editor {
                                        main_state.note_editor.mode
                                    } else {
                                        Mode::Read
                                    })
                                    .set_content(&note.content)
                                    .set_path(note.path.into())
                            })
                            .unwrap_or_default();

                        state.with_main_state(MainState {
                            explorer,
                            note_editor,
                            selected_note,
                            ..*main_state
                        })
                    }
                    _ => state.with_main_state(MainState {
                        explorer,
                        ..*main_state
                    }),
                }
            }
            Message::NoteEditor(message) => {
                let ScreenState::Main(main_state) = screen else {
                    return state;
                };

                let mode = &main_state.note_editor.mode();

                let editor_enabled = self.config.experimental_editor;

                if editor_enabled {
                    match message {
                        note_editor::Message::KeyEvent(key) if *mode == Mode::Edit => {
                            let note_editor = main_state.note_editor.edit(key.into());
                            let selected_note = main_state.selected_note.map(|note| SelectedNote {
                                content: note_editor.content().to_string(),
                                ..note
                            });

                            return state.with_main_state(MainState {
                                note_editor,
                                selected_note,
                                ..*main_state
                            });
                        }
                        note_editor::Message::CursorLeft => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.cursor_left(),
                                ..*main_state
                            })
                        }
                        note_editor::Message::CursorRight => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.cursor_right(),
                                ..*main_state
                            })
                        }
                        note_editor::Message::CursorWordForward => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.cursor_word_forward(),
                                ..*main_state
                            })
                        }
                        note_editor::Message::CursorWordBackward => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.cursor_word_backward(),
                                ..*main_state
                            })
                        }
                        note_editor::Message::Delete => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.delete_char(),
                                ..*main_state
                            })
                        }
                        note_editor::Message::EditMode if *mode != Mode::Edit => {
                            if let Some(selected_note) = &main_state.selected_note {
                                return state.with_main_state(MainState {
                                    active_pane: ActivePane::NoteEditor,
                                    note_editor: main_state
                                        .note_editor
                                        .clone()
                                        .set_content(&selected_note.content)
                                        .set_mode(Mode::Edit),
                                    ..*main_state
                                });
                            } else {
                                return state;
                            }
                        }
                        note_editor::Message::ReadMode if *mode != Mode::Read => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.set_mode(Mode::Read),
                                ..*main_state
                            })
                        }
                        note_editor::Message::ExitMode if *mode == Mode::Read => {
                            return state.with_main_state(MainState {
                                note_editor: main_state.note_editor.set_mode(Mode::View),
                                ..*main_state
                            })
                        }
                        note_editor::Message::ExitMode if *mode == Mode::Edit => {
                            let note_editor = main_state.note_editor.exit_insert();
                            let selected_note = main_state
                                .selected_note
                                .map(|note| SelectedNote {
                                    content: note_editor.content().to_string(),
                                    ..note
                                })
                                .clone();

                            return state.with_main_state(MainState {
                                note_editor: note_editor.set_mode(Mode::View),
                                selected_note,
                                ..*main_state
                            });
                        }
                        note_editor::Message::Save => {
                            let note_editor = main_state.note_editor.save();
                            let selected_note = main_state.selected_note.map(|note| SelectedNote {
                                content: note_editor.content().to_string(),
                                ..note
                            });

                            return state.with_main_state(MainState {
                                selected_note,
                                note_editor,
                                ..*main_state
                            });
                        }
                        _ => {}
                    }
                }

                match message {
                    note_editor::Message::CursorUp => state.with_main_state(MainState {
                        note_editor: main_state.note_editor.cursor_up(),
                        ..*main_state
                    }),
                    note_editor::Message::CursorDown => state.with_main_state(MainState {
                        note_editor: main_state.note_editor.cursor_down(),
                        ..*main_state
                    }),
                    note_editor::Message::ScrollUp(scroll_amount) if *mode != Mode::Edit => state
                        .with_main_state(MainState {
                            note_editor: main_state.note_editor.scroll_up(calc_scroll_amount(
                                scroll_amount,
                                state.screen_size.height.into(),
                            )),
                            ..*main_state
                        }),
                    note_editor::Message::ScrollDown(scroll_amount) if *mode != Mode::Edit => state
                        .with_main_state(MainState {
                            note_editor: main_state.note_editor.scroll_down(calc_scroll_amount(
                                scroll_amount,
                                state.screen_size.height.into(),
                            )),
                            ..*main_state
                        }),
                    note_editor::Message::ToggleExplorer if *mode != Mode::Edit => state
                        .with_main_state(match main_state.explorer.open {
                            true => MainState {
                                explorer: main_state.explorer.toggle(),
                                ..*main_state
                            },
                            false => MainState {
                                active_pane: ActivePane::Explorer,
                                explorer: main_state.explorer.toggle().set_active(true),
                                note_editor: main_state.note_editor.set_active(false),
                                ..*main_state
                            },
                        }),
                    note_editor::Message::SwitchPaneNext => state.with_main_state(MainState {
                        active_pane: ActivePane::Outline,
                        note_editor: main_state.note_editor.set_active(false),
                        outline: main_state.outline.set_active(true),
                        ..*main_state
                    }),
                    note_editor::Message::SwitchPanePrevious => state.with_main_state(MainState {
                        active_pane: ActivePane::Explorer,
                        note_editor: main_state.note_editor.set_active(false),
                        explorer: main_state.explorer.set_active(true),
                        ..*main_state
                    }),
                    note_editor::Message::ScrollUp(_) if *mode == Mode::Edit => state
                        .with_main_state(MainState {
                            note_editor: main_state.note_editor.cursor_up(),
                            ..*main_state
                        }),
                    note_editor::Message::ScrollDown(_) if *mode == Mode::Edit => state
                        .with_main_state(MainState {
                            note_editor: main_state.note_editor.cursor_down(),
                            ..*main_state
                        }),
                    _ => state,
                }
            }
        }
    }

    fn render_splash(&self, area: Rect, buf: &mut Buffer, state: &mut SplashState<'a>) {
        Splash::default().render_ref(area, buf, state)
    }

    fn render_main(&self, area: Rect, buf: &mut Buffer, state: &mut MainState<'a>) {
        let [content, statusbar] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)])
            .horizontal_margin(1)
            .areas(area);

        let (left, right) = if state.explorer.open {
            (Constraint::Length(35), Constraint::Fill(1))
        } else {
            (Constraint::Length(5), Constraint::Fill(1))
        };

        let [explorer_pane, note] = Layout::horizontal([left, right]).areas(content);

        Explorer::new().render(explorer_pane, buf, &mut state.explorer);
        Editor::default().render(note, buf, &mut state.note_editor);

        let (_, counts) = state
            .selected_note
            .clone()
            .map(|note| {
                let content = note.content.as_str();
                (
                    note.name,
                    (WordCount::from(content), CharCount::from(content)),
                )
            })
            .unzip();

        let (word_count, char_count) = counts.unwrap_or_default();

        let mut status_bar_state = StatusBarState::new(
            state.active_pane.into(),
            word_count.into(),
            char_count.into(),
        );

        let status_bar = StatusBar::default();
        status_bar.render_ref(statusbar, buf, &mut status_bar_state);
    }

    fn render_modals(&self, area: Rect, buf: &mut Buffer, state: &mut AppState<'a>) {
        if state.vault_selector_modal.visible {
            VaultSelectorModal::default().render(area, buf, &mut state.vault_selector_modal);
        }

        if state.help_modal.visible {
            HelpModal.render(area, buf, &mut state.help_modal);
        }
    }
}

impl<'a> StatefulWidgetRef for App<'a> {
    type State = AppState<'a>;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        match &mut state.screen {
            ScreenState::Splash(state) => self.render_splash(area, buf, state),
            ScreenState::Main(state) => self.render_main(area, buf, state),
        };

        self.render_modals(area, buf, state)
    }
}
