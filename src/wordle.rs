use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
pub(crate) enum Color {
    #[default]
    Gray,
    Yellow,
    Green,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy, Default)]
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

#[derive(Deserialize)]
pub(crate) struct Puzzle {
    #[serde(rename = "days_since_launch")]
    pub(crate) number: u32,
    #[serde(rename = "solution")]
    pub(crate) word: String,
}

impl Puzzle {
    pub(crate) async fn today() -> anyhow::Result<Self> {
        Puzzle::at(Utc::now()).await
    }

    pub(crate) async fn at(date: DateTime<Utc>) -> anyhow::Result<Self> {
        let date = date.format("%Y-%m-%d");
        let url = format!("https://www.nytimes.com/svc/wordle/v2/{date}.json");
        let res = reqwest::get(url).await?;
        Ok(res.json::<Self>().await?)
    }
}
