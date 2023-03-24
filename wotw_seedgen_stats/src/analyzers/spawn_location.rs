use wotw_seedgen::generator::SeedSpoiler;

use super::Analyzer;

/// Analyzes the spawn location
pub struct SpawnLocationStats;
impl Analyzer for SpawnLocationStats {
    fn title(&self) -> String {
        "Spawn Location".to_string()
    }

    fn analyze(&self, seed: &SeedSpoiler) -> Vec<String> {
        seed.spawns.clone()
    }
}
