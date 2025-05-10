use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, PartialOrd)]
pub(crate) enum Color {
    #[default]
    Gray,
    Yellow,
    Green,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub(crate) struct Letter {
    pub(crate) char: char,
    pub(crate) color: Option<Color>,
}

impl Default for Letter {
    fn default() -> Self {
        Self {
            char: ' ',
            color: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Row {
    pub(crate) letters: [Letter; 5],
}

impl Row {
    pub(crate) fn set_colors(&mut self, word: &str) {
        let mut unused_letters = [true; 5];

        #[allow(clippy::needless_range_loop)]
        for i in 0..5 {
            let char = self.letters[i].char.to_ascii_lowercase();
            if word.chars().nth(i).map(|c| c.to_ascii_lowercase()) == Some(char) {
                unused_letters[i] = false;
                self.letters[i].color = Some(Color::Green)
            }
        }

        for i in 0..5 {
            let char = self.letters[i].char.to_ascii_lowercase();
            if self.letters[i].color.is_none() {
                self.letters[i].color = if let Some((i, _)) = word
                    .chars()
                    .map(|c| c.to_ascii_lowercase())
                    .enumerate()
                    .find(|(i, c)| unused_letters[*i] && *c == char)
                {
                    unused_letters[i] = false;
                    Some(Color::Yellow)
                } else {
                    Some(Color::Gray)
                };
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct GameInfo {
    #[serde(default, rename = "days_since_launch")]
    pub(crate) number: u32,
    #[serde(rename = "solution")]
    pub(crate) word: String,
    #[serde(rename = "print_date")]
    pub(crate) date_string: String,
}

impl GameInfo {
    pub(crate) async fn today() -> anyhow::Result<Self> {
        GameInfo::at(Utc::now()).await
    }

    pub(crate) async fn at(date: DateTime<Utc>) -> anyhow::Result<Self> {
        let date = date.format("%Y-%m-%d");
        let url = format!("https://www.nytimes.com/svc/wordle/v2/{date}.json");
        let res = reqwest::get(url).await?;
        Ok(res.json::<Self>().await?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Game {
    pub(crate) grid: [Row; 6],
    pub(crate) index: (usize, usize),
    pub(crate) info: GameInfo,
}

impl Game {
    pub(crate) fn has_finished(&self) -> bool {
        self.won_in().is_some()
            || !self
                .grid
                .iter()
                .any(|row| row.letters.iter().all(|l| l.color.is_none()))
    }

    pub(crate) fn won_in(&self) -> Option<usize> {
        self.grid
            .iter()
            .enumerate()
            .find(|(_, row)| row.letters.iter().all(|l| l.color == Some(Color::Green)))
            .map(|(i, _)| i + 1)
    }

    pub(crate) fn add_char(&mut self, char: char) {
        if self.has_finished() {
            return;
        }

        if self.index.1 >= 5 {
            return;
        }
        self.grid[self.index.0].letters[self.index.1].char = char;
        self.index.1 += 1;
    }

    pub(crate) fn backspace(&mut self) {
        if self.has_finished() {
            return;
        }

        if self.index.1 == 0 {
            return;
        }
        self.index.1 -= 1;
        self.grid[self.index.0].letters[self.index.1].char = ' ';
    }

    pub(crate) fn submit(&mut self) {
        if self.has_finished() {
            return;
        }

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
            self.grid[self.index.0].set_colors(&self.info.word);
            self.index.0 += 1;
            self.index.1 = 0;
        }
    }
}

impl From<GameInfo> for Game {
    fn from(info: GameInfo) -> Self {
        Self {
            grid: [Row::default(); 6],
            index: (0, 0),
            info,
        }
    }
}
