use crate::state::Mood;
use serde::Deserialize;
use std::sync::OnceLock;

/// The species table, loaded once from species.toml at the crate root.
/// Species are data: name pool, art per mood, reaction lines, rarity tier.
#[derive(Debug, Deserialize)]
pub struct Species {
    pub name: String,
    pub tier: String,
    pub names: Vec<String>,
    pub art: MoodArt,
    pub lines: MoodLines,
}

#[derive(Debug, Deserialize)]
pub struct MoodArt {
    pub idle: String,
    pub working: String,
    pub celebrating: String,
    pub miffed: String,
}

#[derive(Debug, Deserialize)]
pub struct MoodLines {
    pub idle: Vec<String>,
    pub working: Vec<String>,
    pub celebrating: Vec<String>,
    pub miffed: Vec<String>,
}

impl MoodArt {
    pub fn for_mood(&self, mood: Mood) -> &str {
        match mood {
            Mood::Idle => &self.idle,
            Mood::Working => &self.working,
            Mood::Celebrate => &self.celebrating,
            Mood::Miffed => &self.miffed,
        }
    }
}

impl MoodLines {
    pub fn for_mood(&self, mood: Mood) -> &[String] {
        match mood {
            Mood::Idle => &self.idle,
            Mood::Working => &self.working,
            Mood::Celebrate => &self.celebrating,
            Mood::Miffed => &self.miffed,
        }
    }
}

const SPECIES_TOML: &str = include_str!("../species.toml");

#[derive(Debug, Deserialize)]
struct SpeciesFile {
    species: Vec<Species>,
}

pub fn table() -> &'static [Species] {
    static TABLE: OnceLock<Vec<Species>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let parsed: SpeciesFile = toml::from_str(SPECIES_TOML).expect("species.toml must parse");
        assert!(!parsed.species.is_empty(), "species table is empty");
        parsed.species
    })
}

pub fn get(name: &str) -> Option<&'static Species> {
    table().iter().find(|s| s.name == name)
}

/// The first species in the table is the fallback for anything unresolved.
pub fn fallback() -> &'static Species {
    table().first().expect("species table is never empty")
}

/// A state naming an unknown species falls back to the first species.
pub fn resolve(name: &str) -> &'static Species {
    get(name).unwrap_or_else(fallback)
}

/// The species word shown in the status line and the TUI title.
pub fn display_name(name: &str) -> &'static str {
    resolve(name).name.as_str()
}

/// A new hatch picks a species uniformly at random (seeded, so testable).
pub fn pick(seed: u64) -> &'static Species {
    let n = table().len() as u64;
    &table()[(seed % n) as usize]
}

/// Deterministic blurb pick from a mood's line pool; drifts slowly with time
/// so a pool with more than one line gets used.
pub fn pick_line(pool: &[String], now: i64) -> &str {
    if pool.is_empty() {
        return "";
    }
    let idx = ((now / 600).max(0) as usize) % pool.len();
    pool[idx].as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_loads_and_names_are_unique() {
        let t = table();
        // The buddy graduation completes the table: exactly thirty-eight species (plan
        // milestone 5).
        assert_eq!(t.len(), 38, "the full table is thirty-eight species");
        let mut names: Vec<&str> = t.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate species names in table");
    }

    #[test]
    fn every_species_has_complete_art_for_every_mood() {
        for sp in table() {
            for mood in [Mood::Idle, Mood::Working, Mood::Celebrate, Mood::Miffed] {
                let art = sp.art.for_mood(mood);
                assert!(!art.trim().is_empty(), "{} art empty for {}", sp.name, mood.label());
                assert!(art.lines().count() >= 3, "{} art too short for {}", sp.name, mood.label());
            }
        }
    }

    #[test]
    fn name_pools_non_empty() {
        for sp in table() {
            assert!(sp.names.len() >= 8, "{} name pool below the eight-name floor: {}", sp.name, sp.names.len());
            for mood in [Mood::Idle, Mood::Working, Mood::Celebrate, Mood::Miffed] {
                assert!(!sp.lines.for_mood(mood).is_empty(), "{} lines empty for {}", sp.name, mood.label());
            }
        }
    }

    #[test]
    fn no_duplicate_names_across_species() {
        let mut all: Vec<&str> = Vec::new();
        for sp in table() {
            all.extend(sp.names.iter().map(|s| s.as_str()));
        }
        all.sort();
        let n = all.len();
        all.dedup();
        assert_eq!(all.len(), n, "name shared across species pools");
    }

    #[test]
    fn unknown_species_falls_back() {
        assert_eq!(resolve("trilobite").name, "ember");
        assert_eq!(display_name("trilobite"), "ember");
        assert_eq!(display_name("moss"), "moss");
    }

    #[test]
    fn pick_is_uniform_over_the_table() {
        let t = table();
        for seed in 0..(t.len() as u64 * 3) {
            assert!(get(&pick(seed).name).is_some());
        }
        assert_eq!(pick(0).name, t[0].name);
    }
}
