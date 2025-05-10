use std::{array, borrow::Cow, collections::HashSet};

use ratatui::{
    layout::{Constraint, Direction, Flex, Layout},
    prelude::{self, Color, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Bar, BarChart, BarGroup, Block, Padding, Paragraph, Widget},
};

use crate::{stats::Stats, wordle};

impl From<&wordle::Color> for Color {
    fn from(value: &wordle::Color) -> Self {
        match value {
            wordle::Color::Gray => Color::DarkGray,
            wordle::Color::Green => Color::Green,
            wordle::Color::Yellow => Color::Yellow,
        }
    }
}

impl Widget for &wordle::Letter {
    fn render(self, area: Rect, buf: &mut prelude::Buffer)
    where
        Self: Sized,
    {
        let mut block = Block::new()
            .fg(Color::White)
            .padding(Padding::top(area.height / 2));

        if let Some(color) = &self.color {
            block = block.bg(color);
        }

        let text = Paragraph::new(self.char.to_string())
            .bold()
            .centered()
            .block(block);
        text.render(area, buf);
    }
}

impl Widget for &wordle::Row {
    fn render(self, area: Rect, buf: &mut prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::horizontal([Constraint::Percentage(20); 5])
            .flex(Flex::Start)
            .spacing(1);
        for (area, letter) in layout.areas::<5>(area).iter().zip(&self.letters) {
            letter.render(*area, buf);
        }
    }
}

struct KeyboardRow<const N: usize> {
    letters: [wordle::Letter; N],
}

impl<const N: usize> KeyboardRow<N> {
    fn from_chars(chars: [char; N]) -> Self {
        Self {
            letters: chars.map(|c| wordle::Letter {
                char: c,
                color: None,
            }),
        }
    }

    fn set_color(&mut self, char: char, color: Option<wordle::Color>) {
        if let Some(letter) = self.letters.iter_mut().find(|l| l.char == char) {
            if color > letter.color {
                letter.color = color;
            }
        }
    }
}

impl<const N: usize> Widget for &KeyboardRow<N> {
    fn render(self, area: Rect, buf: &mut prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::horizontal([Constraint::Min(1); N]).flex(Flex::Start);
        for (area, letter) in layout.areas::<N>(area).iter().zip(&self.letters) {
            letter.render(*area, buf);
        }
    }
}

pub(crate) struct Keyboard {
    rows: (KeyboardRow<10>, KeyboardRow<9>, KeyboardRow<7>),
}

impl Keyboard {
    pub(crate) fn from_rows(rows: &[wordle::Row]) -> Self {
        let mut keyboard = Self {
            rows: (
                KeyboardRow::from_chars(['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P']),
                KeyboardRow::from_chars(['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L']),
                KeyboardRow::from_chars(['Z', 'X', 'C', 'V', 'B', 'N', 'M']),
            ),
        };

        for row in rows {
            for letter in row.letters {
                keyboard.rows.0.set_color(letter.char, letter.color);
                keyboard.rows.1.set_color(letter.char, letter.color);
                keyboard.rows.2.set_color(letter.char, letter.color);
            }
        }

        keyboard
    }
}

impl Widget for &Keyboard {
    fn render(self, area: Rect, buf: &mut prelude::Buffer)
    where
        Self: Sized,
    {
        let layout = Layout::vertical([Constraint::Min(1); 3]).flex(Flex::Start);
        let [area1, area2, area3] = layout.areas(area);
        self.rows.0.render(area1, buf);
        self.rows.1.render(area2, buf);
        self.rows.2.render(area3, buf);
    }
}

impl Widget for &wordle::Game {
    fn render(self, area: Rect, buf: &mut prelude::Buffer)
    where
        Self: Sized,
    {
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

        let [title_area, game_area, message_area, keyboard_area] = layout.areas(area);
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
            row.render(area, buf);
        }

        Keyboard::from_rows(&self.grid).render(keyboard_area, buf);

        Paragraph::new(format!(
            "Wordle #{} - {}",
            self.info.number, self.info.date_string
        ))
        .bold()
        .centered()
        .render(title_area, buf);

        let message: Cow<str> = if self.has_finished() {
            match self.won_in() {
                Some(1) => "Genius".into(),
                Some(2) => "Magnificent".into(),
                Some(3) => "Impressive".into(),
                Some(4) => "Splendid".into(),
                Some(5) => "Great".into(),
                Some(6) => "Phew".into(),
                None => self.info.word.to_uppercase().into(),
                _ => unreachable!(),
            }
        } else {
            "".into()
        };

        Paragraph::new(message)
            .bold()
            .centered()
            .render(message_area, buf);
    }
}

impl Widget for &Stats {
    fn render(self, area: Rect, buf: &mut prelude::Buffer)
    where
        Self: Sized,
    {
        let [general_area, chart_area] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(1)])
                .spacing(2)
                .areas(area);

        let [played_area, percent_area] =
            Layout::horizontal([Constraint::Percentage(50); 2]).areas(general_area);

        let played_block = Paragraph::new(format!("{}", self.attempted))
            .bold()
            .centered()
            .block(Block::new().title_bottom(Line::from("Played").not_bold().centered()));

        let win_percentage =
            (self.won.iter().sum::<usize>() as f32 / self.attempted as f32 * 100.0).round();

        let percent_block = Paragraph::new(format!("{win_percentage}"))
            .bold()
            .centered()
            .block(Block::new().title_bottom(Line::from("Win %").not_bold().centered()));

        played_block.render(played_area, buf);
        percent_block.render(percent_area, buf);

        let max = *self.won.iter().max().unwrap();
        let max_indices = self
            .won
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (*c == max).then_some(i))
            .collect::<HashSet<_>>();

        let mut bars: [Bar; 6] = array::from_fn(|_| Bar::default());

        for (i, bar) in bars.iter_mut().enumerate() {
            *bar = Bar::default()
                .label(format!("{}", i + 1).into())
                .value(self.won[i] as u64)
                .style(if max_indices.contains(&i) {
                    Style::new().green()
                } else {
                    Style::new().dark_gray()
                })
                .value_style(if max_indices.contains(&i) {
                    Style::new().bold().white().on_green()
                } else {
                    Style::new().bold().white().on_dark_gray()
                })
        }

        let chart = BarChart::default()
            .direction(Direction::Horizontal)
            .bar_width(1)
            .bar_gap(0)
            .label_style(Style::new().white())
            .data(BarGroup::default().bars(&bars))
            .max(max as u64);

        let chart_block = Block::new()
            .title(Line::from("Guess Distribution").bold().centered())
            .padding(Padding::vertical(1));
        let chart_block_area = chart_block.inner(chart_area);

        chart_block.render(chart_area, buf);
        chart.render(chart_block_area, buf);
    }
}
