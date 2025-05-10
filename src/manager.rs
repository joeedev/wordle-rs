use std::ops::{Deref, DerefMut};

use chrono::{DateTime, Duration, Utc};

use crate::{SaveData, wordle};

pub(crate) struct GameManager {
    latest_game_number: u32,
    game: wordle::Game,
    pub(crate) date: DateTime<Utc>,
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
            latest_game_number: game.info.number,
            game,
            date: Utc::now(),
            save_data,
        })
    }

    pub(crate) fn save(&mut self) {
        self.save_data.save(&self.game);
    }

    async fn offset_by(&mut self, offset: i32) {
        let new_date = self.date + Duration::days(offset as i64);
        if new_date.date_naive() > Utc::now().date_naive() {
            return;
        }

        self.date = new_date;

        if let Some(game) = self
            .save_data
            .load((self.game.info.number as i32 + offset) as u32)
        {
            game.clone_into(&mut self.game);
        } else if let Ok(game) = wordle::GameInfo::at(self.date)
            .await
            .map(|info| -> wordle::Game { info.into() })
        {
            game.clone_into(&mut self.game);
        }
    }

    pub(crate) async fn next(&mut self) {
        self.offset_by(1).await;
    }

    pub(crate) async fn previous(&mut self) {
        self.offset_by(-1).await;
    }

    pub(crate) async fn first(&mut self) {
        self.offset_by(-(self.game.info.number as i32)).await;
    }

    pub(crate) async fn last(&mut self) {
        self.offset_by(self.latest_game_number as i32 - self.game.info.number as i32)
            .await;
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
