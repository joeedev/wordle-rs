use ratatui::{
    layout::{Constraint, Flex, Layout},
    prelude::{self, Color, Rect},
    style::Stylize,
    widgets::{Block, Padding, Paragraph, Widget},
};

use crate::wordle;

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
