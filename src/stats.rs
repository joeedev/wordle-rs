use crate::SaveData;

#[derive(Default)]
pub(crate) struct Stats {
    pub(crate) attempted: usize,
    pub(crate) won: [usize; 6],
}

impl SaveData {
    pub(crate) fn stats(&self) -> Stats {
        let mut stats = Stats::default();

        for game in self.games().filter(|game| game.has_finished()) {
            stats.attempted += 1;
            if let Some(guesses) = game.won_in() {
                stats.won[guesses - 1] += 1;
            }
        }

        stats
    }
}
