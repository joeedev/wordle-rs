use std::ops::{Deref, DerefMut};

use chrono::{Duration, NaiveDate, Utc};

use crate::{SaveData, wordle};

const FIRST_WORDLE_DATE: NaiveDate = NaiveDate::from_ymd_opt(2021, 6, 19).unwrap();

fn date_to_wordle_number(date: NaiveDate) -> u32 {
    (date - FIRST_WORDLE_DATE).num_days().try_into().unwrap()
}

pub(crate) struct GameManager {
    game: wordle::Game,
    pub(crate) date: NaiveDate,
    pub(crate) save_data: SaveData,
}

impl GameManager {
    pub(crate) async fn new() -> anyhow::Result<Self> {
        let mut game: wordle::Game = wordle::GameInfo::today().await?.into();
        let save_data = SaveData::from_file().unwrap_or(SaveData::new());

        if let Some(saved_game) = save_data.load(game.info.number) {
            saved_game.clone_into(&mut game);
        }

        Ok(Self {
            game,
            date: Utc::now().date_naive(),
            save_data,
        })
    }

    pub(crate) fn save(&mut self) {
        self.save_data.save(&self.game);
    }

    async fn goto(&mut self, date: NaiveDate) {
        self.date = date;

        if let Some(game) = self.save_data.load(date_to_wordle_number(date)) {
            game.clone_into(&mut self.game);
        } else if let Ok(game) = wordle::GameInfo::at(self.date)
            .await
            .map(|info| -> wordle::Game { info.into() })
        {
            game.clone_into(&mut self.game);
        }
    }

    async fn offset_by(&mut self, offset: i32) {
        let new_date = self.date + Duration::days(offset as i64);
        if new_date < FIRST_WORDLE_DATE || new_date > Utc::now().date_naive() {
            return;
        }
        self.goto(new_date).await;
    }

    pub(crate) async fn next(&mut self) {
        self.offset_by(1).await;
    }

    pub(crate) async fn previous(&mut self) {
        self.offset_by(-1).await;
    }

    pub(crate) async fn first(&mut self) {
        self.goto(FIRST_WORDLE_DATE).await;
    }

    pub(crate) async fn last(&mut self) {
        self.goto(Utc::now().date_naive()).await;
    }
}

impl Deref for GameManager {
    type Target = wordle::Game;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl DerefMut for GameManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game
    }
}
