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
            Mood::Sleeping | Mood::Resting | Mood::Petted => &self.idle,
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
            Mood::Sleeping | Mood::Resting | Mood::Petted => &self.idle,
        }
    }
}

const SPECIES_TOML: &str = include_str!("../species.toml");

#[derive(Debug, Deserialize)]
struct SpeciesFile {
    species: Vec<Species>,
}

/// One spoken line for a mood, rotated by seed so back-to-back calls
/// usually differ. An empty pool returns "" and callers skip speaking
/// rather than print silence.
pub fn speak(sp: &Species, mood: Mood, seed: u64) -> &str {
    let pool = sp.lines.for_mood(mood);
    if pool.is_empty() {
        return "";
    }
    &pool[(seed % pool.len() as u64) as usize]
}

/// The species the binary ships with, parsed fresh each call (tests use it
/// directly; runtime goes through `table`, which merges packs on top).
pub fn builtin_table() -> Vec<Species> {
    let parsed: SpeciesFile = toml::from_str(SPECIES_TOML).expect("species.toml must parse");
    assert!(!parsed.species.is_empty(), "species table is empty");
    parsed.species
}

pub fn table() -> &'static [Species] {
    static TABLE: OnceLock<Vec<Species>> = OnceLock::new();
    TABLE.get_or_init(|| merge(builtin_table(), &load_pack_files()))
}

/// Runtime species packs: every *.toml in $HATCH_HOME/packs, sorted by file
/// name so the load order is stable. Missing dir means no packs.
fn load_pack_files() -> Vec<(String, String)> {
    let dir = crate::state::config_dir().join("packs");
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return out;
    };
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();
    for p in paths {
        if p.extension().map(|e| e == "toml").unwrap_or(false) {
            if let Ok(text) = std::fs::read_to_string(&p) {
                out.push((p.display().to_string(), text));
            }
        }
    }
    out
}

/// Merge pack files into the builtin table. A pack species carrying the name
/// of a builtin replaces it; a new name appends. A file that does not parse
/// is skipped whole: a bad pack must never take the buddy down.
fn merge(builtins: Vec<Species>, packs: &[(String, String)]) -> Vec<Species> {
    let mut table = builtins;
    for (_, text) in packs {
        let Ok(parsed) = toml::from_str::<SpeciesFile>(text) else {
            continue;
        };
        for sp in parsed.species {
            if let Some(slot) = table.iter_mut().find(|t| t.name == sp.name) {
                *slot = sp;
            } else {
                table.push(sp);
            }
        }
    }
    table
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

/// Rarity tiers and their hatch weights, out of 100. Introduced in milestone
/// 6; every species carries its tier in the species table.
pub const TIER_WEIGHTS: [(&str, u64); 5] = [
    ("common", 50),
    ("uncommon", 25),
    ("rare", 15),
    ("epic", 8),
    ("legendary", 2),
];

pub const TIER_WEIGHT_TOTAL: u64 = 100;

/// A new hatch rolls a rarity tier first, then a species within that tier,
/// all seeded so tests can reproduce every roll.
pub fn pick(seed: u64) -> &'static Species {
    let roll = seed % TIER_WEIGHT_TOTAL;
    let mut acc = 0u64;
    for (tier, weight) in TIER_WEIGHTS {
        acc += weight;
        if roll < acc {
            let group: Vec<&'static Species> =
                table().iter().filter(|s| s.tier == tier).collect();
            if !group.is_empty() {
                return group[((seed >> 8) as usize) % group.len()];
            }
        }
    }
    // Defensive fallback: a tier with no species would land here.
    let t = table();
    &t[(seed as usize) % t.len()]
}

/// `tb dex` rows: discovered species show name, species and tier; undiscovered
/// ones hide everything behind ???. The first row is the summary counter.
pub fn dex_rows(discovered: &[String]) -> Vec<String> {
    let t = table();
    let mut rows = Vec::with_capacity(t.len() + 1);
    rows.push(format!("{}/{} discovered", discovered.len(), t.len()));
    for sp in t {
        if discovered.iter().any(|d| d.as_str() == sp.name) {
            rows.push(format!("  {} the {} [{}]", sp.names[0], sp.name, sp.tier));
        } else {
            rows.push("  ??? the ???".to_string());
        }
    }
    rows
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
        // The buddy graduation completes the table: exactly thirty-eight built-in
        // species (plan milestone 5). Runtime packs may add or override on top.
        assert_eq!(builtin_table().len(), 38, "the full table is thirty-eight species");
        let mut names: Vec<&str> = t.iter().map(|s| s.name.as_str()).collect();
        names.sort();
        let n = names.len();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate species names in table");
    }

    #[test]
    fn pack_override_replaces_builtin() {
        let pack = r#"
[[species]]
name = "ember"
tier = "legendary"
names = ["Ember Prime"]
[species.art]
idle = "x"
working = "x"
celebrating = "x"
miffed = "x"
[species.lines]
idle = []
working = []
celebrating = []
miffed = []
"#;
        let merged = merge(builtin_table(), &[("pack.toml".into(), pack.to_string())]);
        let embers: Vec<_> = merged.iter().filter(|s| s.name == "ember").collect();
        assert_eq!(embers.len(), 1, "override replaces, never duplicates");
        assert_eq!(embers[0].tier, "legendary");
        assert_eq!(embers[0].names, vec!["Ember Prime"]);
        assert_eq!(merged.len(), 38, "override keeps the table the same size");
    }

    #[test]
    fn pack_new_species_appends() {
        let pack = r#"
[[species]]
name = "newt"
tier = "common"
names = ["Newton"]
[species.art]
idle = "x"
working = "x"
celebrating = "x"
miffed = "x"
[species.lines]
idle = []
working = []
celebrating = []
miffed = []
"#;
        let merged = merge(builtin_table(), &[("pack.toml".into(), pack.to_string())]);
        assert_eq!(merged.len(), 39, "a new pack species is appended");
        assert!(merged.iter().any(|s| s.name == "moth"));
    }

    #[test]
    fn broken_pack_is_skipped_whole() {
        let merged = merge(builtin_table(), &[("bad.toml".into(), "not toml [[[".to_string())]);
        assert_eq!(merged.len(), 38, "a bad pack never takes the buddy down");
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
    fn pick_is_weighted_by_tier() {
        let t = table();
        let trials = 100_000u64; // a multiple of TIER_WEIGHT_TOTAL: exact counts
        let mut by_tier: std::collections::HashMap<&str, u64> =
            std::collections::HashMap::new();
        let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for seed in 0..trials {
            let sp = pick(seed);
            *by_tier.entry(sp.tier.as_str()).or_insert(0) += 1;
            seen.insert(sp.name.as_str());
        }
        assert_eq!(seen.len(), t.len(), "every species must be reachable");
        for (tier, weight) in TIER_WEIGHTS {
            let expected = trials * weight / TIER_WEIGHT_TOTAL;
            let got = *by_tier.get(tier).unwrap_or(&0);
            assert_eq!(got, expected, "tier {} odds off", tier);
        }
    }

    #[test]
    fn every_species_carries_a_known_tier() {
        let valid: std::collections::HashSet<&str> =
            TIER_WEIGHTS.iter().map(|(t, _)| *t).collect();
        for sp in table() {
            assert!(
                valid.contains(sp.tier.as_str()),
                "unknown tier {} on {}",
                sp.tier,
                sp.name
            );
        }
        for (tier, _) in TIER_WEIGHTS {
            assert!(
                table().iter().any(|s| s.tier == tier),
                "tier {} has no species",
                tier
            );
        }
    }

    #[test]
    fn dex_hides_undiscovered_species() {
        let t = table();
        let all: Vec<String> = t.iter().map(|s| s.name.clone()).collect();
        let rows = dex_rows(&all);
        assert_eq!(rows.len(), t.len() + 1);
        assert!(rows[0].starts_with(&format!("{}/{}", t.len(), t.len())));
        assert!(rows.iter().all(|r| !r.contains("???")));
        let none = dex_rows(&[]);
        assert_eq!(none[0].starts_with("0/"), true);
        assert_eq!(none.iter().filter(|r| r.contains("???")).count(), t.len());
    }

    #[test]
    fn speak_rotates_within_pools() {
        let sp = resolve("overlord");
        let a = speak(sp, Mood::Idle, 0);
        let b = speak(sp, Mood::Idle, 1);
        assert_ne!(a, b, "seed should move through the pool");
        assert_eq!(speak(sp, Mood::Idle, 0), a, "same seed, same line");
    }

    #[test]
    fn every_species_can_speak_in_every_mood() {
        // Table integrity: no empty pools, so every voice path finds a line
        // and never prints silence.
        for s in table() {
            for m in [Mood::Idle, Mood::Working, Mood::Celebrate, Mood::Miffed] {
                assert!(!speak(s, m, 0).is_empty(), "{} has an empty pool", s.name);
            }
        }
    }
}
