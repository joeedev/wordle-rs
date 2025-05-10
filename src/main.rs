mod widget;
mod wordle;

use chrono::{DateTime, Days, Utc};
use crossterm::event::{self, Event, KeyModifiers};
use ratatui::Frame;

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
    Quit,
}

struct Model {
    date: DateTime<Utc>,
    game: wordle::Game,
    running_state: RunningState,
}

impl Model {
    fn new(info: wordle::GameInfo) -> Self {
        Self {
            date: Utc::now(),
            game: info.into(),
            running_state: RunningState::Running,
        }
    }

    async fn update_game_info(&mut self) {
        if let Ok(info) = wordle::GameInfo::at(self.date).await {
            self.game = info.into();
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
                let new_date = self.date + Days::new(1);
                if new_date.date_naive() <= Utc::now().date_naive() {
                    self.date = new_date;
                    self.update_game_info().await;
                }
            }
            Message::Previous => {
                let new_date = self.date - Days::new(1);
                if new_date.date_naive() <= Utc::now().date_naive() {
                    self.date = new_date;
                    self.update_game_info().await;
                }
            }
            Message::Quit => {
                self.running_state = RunningState::Done;
            }
        }
    }

    fn view(&self, frame: &mut Frame) {
        frame.render_widget(&self.game, frame.area());
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let puzzle = wordle::GameInfo::today().await;

    let mut terminal = ratatui::init();
    let mut model = Model::new(puzzle.unwrap());

    while model.running_state == RunningState::Running {
        terminal
            .draw(|f| model.view(f))
            .expect("failed to draw frame");

        let message = match event::read().expect("failed to read event") {
            Event::Key(e) if e.code.is_char('c') && e.modifiers.contains(KeyModifiers::CONTROL) => {
                Some(Message::Quit)
            }
            Event::Key(e) if e.code.is_backspace() => Some(Message::Backspace),
            Event::Key(e) if e.code.is_enter() => Some(Message::Submit),
            Event::Key(e) if e.code.is_left() => Some(Message::Previous),
            Event::Key(e) if e.code.is_right() => Some(Message::Next),
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
