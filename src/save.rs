use std::{collections::HashMap, fs, path::PathBuf, sync::LazyLock};

use anyhow::Context;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::wordle;

static SAVE_PATH: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    ProjectDirs::from("dev", "joee", "wordle").map(|dirs| dirs.data_dir().to_path_buf())
});

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SaveData {
    map: HashMap<u32, wordle::Game>,
}

impl SaveData {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(crate) fn from_file() -> anyhow::Result<Self> {
        let path = SAVE_PATH
            .as_ref()
            .with_context(|| "Failed to find save directory")?
            .join("save.dat");
        let file = fs::File::open(path).with_context(|| "Failed to open file")?;
        let (result, ..) =
            postcard::from_io((file, &mut [0; 2048])).with_context(|| "Failed to decode")?;
        Ok(result)
    }

    fn save_to_file(&self) -> anyhow::Result<()> {
        let path = SAVE_PATH
            .as_ref()
            .with_context(|| "Failed to find save directory")?;

        let _ = fs::create_dir_all(path);
        let file = fs::File::create(path.join("save.dat"))?;

        postcard::to_io(self, file)?;

        Ok(())
    }

    pub(crate) fn games(&self) -> impl Iterator<Item = &wordle::Game> {
        self.map.values()
    }

    pub(crate) fn save(&mut self, game: &wordle::Game) {
        self.map.insert(game.info.number, game.clone());
    }

    pub(crate) fn load(&self, number: u32) -> Option<&wordle::Game> {
        self.map.get(&number)
    }
}

impl Drop for SaveData {
    fn drop(&mut self) {
        let _ = self.save_to_file();
    }
}
