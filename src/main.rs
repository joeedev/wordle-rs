mod manager;
mod save;
mod stats;
mod widget;
mod wordle;

use crossterm::event::{self, Event, KeyModifiers};
use manager::GameManager;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Block, Clear, Padding},
};
use save::SaveData;
use stats::Stats;

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

#[derive(Default, PartialEq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

enum Message {
    Letter(char),
    Backspace,
    Submit,

    Next,
    Previous,
    First,
    Last,

    Stats,
    Escape,
    Quit,
}

struct Model {
    game: GameManager,
    stats: Option<Stats>,
    running_state: RunningState,
}

impl Model {
    async fn new() -> Self {
        Self {
            game: GameManager::new().await.expect("game should be fetched"),
            stats: None,
            running_state: RunningState::Running,
        }
    }

    async fn update(&mut self, msg: Message) {
        match msg {
            Message::Letter(char) => {
                self.game.add_char(char);
            }
            Message::Backspace => {
                self.game.backspace();
            }
            Message::Submit => {
                self.game.submit();
            }

            Message::Next => {
                self.game.next().await;
            }
            Message::Previous => {
                self.game.previous().await;
            }

            Message::First => {
                self.game.first().await;
            }
            Message::Last => {
                self.game.last().await;
            }

            Message::Stats => {
                if self.stats.is_some() {
                    self.stats = None;
                } else {
                    self.stats = Some(self.game.stats());
                }
            }
            Message::Escape => {
                self.stats = None;
            }

            Message::Quit => {
                self.running_state = RunningState::Done;
            }
        }

        self.game.save();
    }

    fn view(&self, frame: &mut Frame) {
        let game: &wordle::Game = &self.game;
        frame.render_widget(game, frame.area());

        if let Some(stats) = &self.stats {
            let block = Block::bordered()
                .title_top(Line::from(" Statistics ").bold().centered())
                .padding(Padding::uniform(1));

            let area = center(frame.area(), Constraint::Max(50), Constraint::Max(18));

            frame.render_widget(Clear, area);
            frame.render_widget(&block, area);
            frame.render_widget(stats, block.inner(area));
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut terminal = ratatui::init();
    let mut model = Model::new().await;

    while model.running_state == RunningState::Running {
        terminal
            .draw(|f| model.view(f))
            .expect("failed to draw frame");

        let message = match event::read().expect("failed to read event") {
            Event::Key(e) if e.code.is_backspace() => Some(Message::Backspace),
            Event::Key(e) if e.code.is_enter() => Some(Message::Submit),

            Event::Key(e) if e.code.is_left() && e.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::First)
            }
            Event::Key(e) if e.code.is_right() && e.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::Last)
            }

            Event::Key(e) if e.code.is_left() => Some(Message::Previous),
            Event::Key(e) if e.code.is_right() => Some(Message::Next),

            Event::Key(e) if e.code.is_char('c') && e.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::Quit)
            }

            Event::Key(e) if e.code.is_char('?') => Some(Message::Stats),
            Event::Key(e) if e.code.is_esc() => Some(Message::Escape),

            Event::Key(e) => match e.code.as_char() {
                Some(c) if c.is_alphabetic() => Some(Message::Letter(c.to_ascii_uppercase())),
                _ => None,
            },

            _ => None,
        };

        if let Some(message) = message {
            model.update(message).await;
        }
    }
    ratatui::restore();
}
