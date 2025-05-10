mod widget;
mod wordle;

use std::borrow::Cow;

use chrono::{DateTime, Days, Utc};
use crossterm::event::{self, Event, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::style::Stylize;
use ratatui::widgets::Paragraph;
use widget::Keyboard;

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
    grid: [wordle::Row; 6],
    index: (usize, usize),
    date: DateTime<Utc>,
    puzzle: wordle::Puzzle,
    running_state: RunningState,
}

impl Model {
    fn new(puzzle: wordle::Puzzle) -> Self {
        Self {
            grid: [wordle::Row::default(); 6],
            index: (0, 0),
            date: Utc::now(),
            puzzle,
            running_state: RunningState::Running,
        }
    }

    fn has_finished(&self) -> bool {
        self.won_in().is_some()
            || !self
                .grid
                .iter()
                .any(|row| row.letters.iter().all(|l| l.color.is_none()))
    }

    fn won_in(&self) -> Option<usize> {
        self.grid
            .iter()
            .enumerate()
            .find(|(_, row)| {
                row.letters
                    .iter()
                    .all(|l| l.color == Some(wordle::Color::Green))
            })
            .map(|(i, _)| i + 1)
    }

    async fn update_puzzle(&mut self) {
        if let Ok(puzzle) = wordle::Puzzle::at(self.date).await {
            self.puzzle = puzzle;
            self.grid = [wordle::Row::default(); 6];
            self.index = (0, 0);
        }
    }

    async fn update(&mut self, msg: Message) {
        let has_finished = self.has_finished();

        match msg {
            Message::Letter(char) if !has_finished => {
                if self.index.1 >= 5 {
                    return;
                }
                self.grid[self.index.0].letters[self.index.1].char = char;
                self.index.1 += 1;
            }
            Message::Backspace if !has_finished => {
                if self.index.1 == 0 {
                    return;
                }
                self.index.1 -= 1;
                self.grid[self.index.0].letters[self.index.1].char = ' ';
            }
            Message::Submit if !has_finished => {
                if self.index.1 < 5 {
                    return;
                }

                let word = self.grid[self.index.0]
                    .letters
                    .iter()
                    .map(|l| l.char)
                    .collect::<String>()
                    .to_lowercase();

                if include_str!("./wordlist.txt").lines().any(|w| w == word) {
                    self.grid[self.index.0].set_colors(&self.puzzle.word);
                    self.index.0 += 1;
                    self.index.1 = 0;
                }
            }
            Message::Next => {
                let new_date = self.date + Days::new(1);
                if new_date.date_naive() <= Utc::now().date_naive() {
                    self.date = new_date;
                    self.update_puzzle().await;
                }
            }
            Message::Previous => {
                let new_date = self.date - Days::new(1);
                if new_date.date_naive() <= Utc::now().date_naive() {
                    self.date = new_date;
                    self.update_puzzle().await;
                }
            }
            Message::Quit => {
                self.running_state = RunningState::Done;
            }
            _ => {}
        }
    }

    fn view(&self, frame: &mut Frame) {
        let layout = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .flex(Flex::Start);

        let game_layout = Layout::horizontal([Constraint::Length(29)])
            .flex(Flex::Center)
            .spacing(2);

        let keyboard_layout = Layout::horizontal([Constraint::Length(40)])
            .flex(Flex::Center)
            .spacing(2);

        let [title_area, game_area, message_area, keyboard_area] = layout.areas(frame.area());
        let [game_area] = game_layout.areas(game_area);
        let [keyboard_area] = keyboard_layout.areas(keyboard_area);

        let grid_layout = Layout::vertical([Constraint::Length(3); 6])
            .flex(Flex::Start)
            .spacing(1);

        for (area, row) in grid_layout
            .areas::<6>(game_area)
            .into_iter()
            .zip(&self.grid)
        {
            frame.render_widget(row, area);
        }

        frame.render_widget(&Keyboard::from_rows(&self.grid), keyboard_area);

        frame.render_widget(
            Paragraph::new(format!("Wordle #{}", self.puzzle.number))
                .bold()
                .centered(),
            title_area,
        );

        let message: Cow<str> = if self.has_finished() {
            match self.won_in() {
                Some(1) => "Genius".into(),
                Some(2) => "Magnificent".into(),
                Some(3) => "Impressive".into(),
                Some(4) => "Splendid".into(),
                Some(5) => "Great".into(),
                Some(6) => "Phew".into(),
                None => self.puzzle.word.to_uppercase().into(),
                _ => unreachable!(),
            }
        } else {
            "".into()
        };

        frame.render_widget(Paragraph::new(message).bold().centered(), message_area);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let puzzle = wordle::Puzzle::today().await;

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
