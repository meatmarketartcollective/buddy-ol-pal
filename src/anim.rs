//! Procedural animation for the 7-state matrix: every mood gets frames
//! derived from its base art, and at high chaos the buddy occasionally
//! throws a random glitch frame instead of the normal one. Glitching is
//! never constant: the chance and the corruption both rise with chaos, but
//! between bursts the normal art shows.

use crate::state::{Mood, GLITCH_THRESHOLD};


/// Ticks the hatch/rehatch animation runs before the normal art takes over.
pub const HATCH_TICKS: u64 = 12;

/// Egg frames for the hatch animation. The popup never resizes mid-hatch:
/// every frame is padded to the species art's line count and width, and the
/// final step is the buddy's own art.
const EGG_WIDE: [&str; 4] = [
    "   .---.  ",
    "  (     ) ",
    "  (     ) ",
    "   '---'  ",
];
const EGG_WIDE_TILT: [&str; 4] = [
    "    .---. ",
    "   (     )",
    "   (     )",
    "    '---' ",
];
const EGG_WIDE_CRACKED: [&str; 4] = [
    "   .---.  ",
    "  ( \\ / ) ",
    "  (  X  ) ",
    "   '---'  ",
];
const EGG_WIDE_OPEN: [&str; 4] = [
    "   .---.  ",
    "  ( o o ) ",
    "  (_____) ",
    "          ",
];
const EGG_SMALL: [&str; 4] = [
    " .--. ",
    "(    )",
    "(    )",
    " '--' ",
];
const EGG_SMALL_TILT: [&str; 4] = [
    "  --  ",
    "(    )",
    "(    )",
    " '--' ",
];
const EGG_SMALL_CRACKED: [&str; 4] = [
    " .--. ",
    "( X  )",
    "( /\\ )",
    " '--' ",
];
const EGG_WIDE3: [&str; 3] = ["   .---.  ", "  (     ) ", "   '---'  "];
const EGG_WIDE3_TILT: [&str; 3] = ["    .---. ", "   (     )", "    '---' "];
const EGG_WIDE3_CRACKED: [&str; 3] = ["   .---.  ", "  ( X / ) ", "   '---'  "];
const EGG_WIDE3_OPEN: [&str; 3] = ["   .---.  ", "  ( o o ) ", "   '---'  "];
const EGG_SMALL3: [&str; 3] = [" .--. ", "(    )", " '--' "];
const EGG_SMALL3_TILT: [&str; 3] = [" ,--. ", "(    )", " `--' "];
const EGG_SMALL3_CRACKED: [&str; 3] = [" .--. ", "( X/ )", " '--' "];
const EGG_SMALL3_OPEN: [&str; 3] = [" .--. ", "( o o)", " '--' "];
const EGG_SMALL_OPEN: [&str; 4] = [
    " .--. ",
    "( o o)",
    "(____)",
    "      ",
];

fn center_block(rows: &[&str], h: usize, w: usize) -> String {
    let top = (h.saturating_sub(rows.len())) / 2;
    let mut out = String::new();
    for i in 0..h {
        if i < top || i >= top + rows.len() {
            out.push_str(&" ".repeat(w));
        } else {
            let row = rows[i - top];
            let len = row.chars().count();
            let pad = w.saturating_sub(len) / 2;
            let line = format!(
                "{}{}{}",
                " ".repeat(pad),
                row,
                " ".repeat(w.saturating_sub(pad + len))
            );
            out.push_str(&line);
        }
        out.push('\n');
    }
    out
}

/// One frame of the hatch (or rehatch) animation. Steps: the egg sits and
/// wobbles, then cracks, then the shell opens on a peeking buddy, then the
/// species art itself. Sized to the base art box so the popup holds still.
pub fn hatch_frame(base: &str, step: u64) -> String {
    if step >= HATCH_TICKS {
        return base.to_string();
    }
    let h = base.lines().count().max(1);
    let w = base.lines().map(|l| l.chars().count()).max().unwrap_or(1);
    // An art box too small to hold an egg simply skips the animation.
    if w < 6 || h < 3 {
        return base.to_string();
    }
    let patterns: &[&str] = match (w >= 10, h) {
        (true, 4..) => match step {
            0..=3 => {
                if step % 2 == 0 {
                    &EGG_WIDE
                } else {
                    &EGG_WIDE_TILT
                }
            }
            4..=7 => &EGG_WIDE_CRACKED,
            _ => &EGG_WIDE_OPEN,
        },
        (false, 4..) => match step {
            0..=3 => {
                if step % 2 == 0 {
                    &EGG_SMALL
                } else {
                    &EGG_SMALL_TILT
                }
            }
            4..=7 => &EGG_SMALL_CRACKED,
            _ => &EGG_SMALL_OPEN,
        },
        (true, 3) => match step {
            0..=3 => {
                if step % 2 == 0 {
                    &EGG_WIDE3
                } else {
                    &EGG_WIDE3_TILT
                }
            }
            4..=7 => &EGG_WIDE3_CRACKED,
            _ => &EGG_WIDE3_OPEN,
        },
        (false, 3) => match step {
            0..=3 => {
                if step % 2 == 0 {
                    &EGG_SMALL3
                } else {
                    &EGG_SMALL3_TILT
                }
            }
            4..=7 => &EGG_SMALL3_CRACKED,
            _ => &EGG_SMALL3_OPEN,
        },
        _ => return base.to_string(),
    };
    center_block(patterns, h, w)
}

const GLITCH_CHARS: &[char] = &['#', '%', '?', '!', '/', '\\', '~', '^', '='];

/// SplitMix64: tiny deterministic hash for frame rolls.
fn hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

/// Chance (out of 100) that this tick glitches, rising with chaos. Chaos 50
/// barely flickers; 100 glitches most ticks but never all of them.
pub fn glitch_chance(chaos: i64) -> i64 {
    if chaos < GLITCH_THRESHOLD {
        0
    } else {
        ((chaos - GLITCH_THRESHOLD + 5) * 2).min(70)
    }
}

/// Share of non-space characters a glitch frame corrupts, rising with chaos.
pub fn glitch_severity(chaos: i64) -> i64 {
    (5 + (chaos - GLITCH_THRESHOLD)).clamp(1, 45)
}

fn closed_eyes(art: &str) -> String {
    art.chars()
        .map(|c| match c {
            'o' | 'O' | '0' | '\u{25C9}' | '\u{25CF}' | '\u{25D5}' | '\u{2665}' => '-',
            _ => c,
        })
        .collect()
}

fn half_lids(art: &str) -> String {
    art.chars()
        .map(|c| match c {
            'o' | 'O' | '0' | '\u{25C9}' | '\u{25CF}' | '\u{25D5}' | '\u{2665}' => '.',
            _ => c,
        })
        .collect()
}

fn zzz_line(tick: u64) -> String {
    format!("{}z Z Z", " ".repeat(3 + (tick % 3) as usize))
}

/// A single corrupted frame. Spaces and newlines stay intact so the box
/// shape never shifts; only visible characters scramble.
pub fn glitch_frame(art: &str, seed: u64, chaos: i64) -> String {
    let severity = glitch_severity(chaos) as u64;
    let mut out = String::with_capacity(art.len());
    let mut cursor = seed;
    for line in art.split_inclusive('\n') {
        for c in line.chars() {
            cursor = hash(cursor);
            if c != ' ' && c != '\n' && cursor % 100 < severity {
                out.push(GLITCH_CHARS[(cursor as usize) % GLITCH_CHARS.len()]);
            } else {
                out.push(c);
            }
        }
    }
    out
}

/// The frame to show right now: the mood's base art animated for its state,
/// and, at high chaos, occasionally replaced by a random glitch frame.
pub fn frame(mood: Mood, base: &str, tick: u64, chaos: i64) -> String {
    let animated = match mood {
        Mood::Sleeping => format!("{}\n{}", closed_eyes(base), zzz_line(tick)),
        Mood::Resting => half_lids(base),
        Mood::Petted => {
            let hearts = if tick % 2 == 0 { "   <3" } else { "  <333" };
            format!("{}\n{}", base, hearts)
        }
        Mood::Working => {
            let spin = ['|', '/', '-', '\\'][(tick % 4) as usize];
            format!("{}\n   {} running", base, spin)
        }
        Mood::Idle => {
            if tick % 4 == 3 {
                closed_eyes(base)
            } else {
                base.to_string()
            }
        }
        _ => base.to_string(),
    };
    let chance = glitch_chance(chaos);
    if chance > 0 {
        let roll = (hash(tick.wrapping_mul(2_654_435_761) ^ (chaos as u64).wrapping_mul(97)) % 100) as i64;
        if roll < chance {
            let seed = hash(tick ^ (chaos as u64).wrapping_mul(1_000_003));
            return glitch_frame(&animated, seed, chaos);
        }
    }
    animated
}

#[cfg(test)]
mod tests {
    use super::*;

    const ART: &str = "  o   o  \n  (o.o)  \n";

    #[test]
    fn sleeping_frames_cycle_the_zzz_drift() {
        assert_ne!(frame(Mood::Sleeping, ART, 0, 0), frame(Mood::Sleeping, ART, 1, 0));
        assert!(frame(Mood::Sleeping, ART, 0, 0).contains("z Z Z"));
        // Asleep means eyes closed in every frame.
        assert!(!frame(Mood::Sleeping, ART, 5, 0).contains('o'));
    }

    #[test]
    fn idle_blinks_every_fourth_tick() {
        assert_eq!(frame(Mood::Idle, ART, 0, 0), ART);
        assert_ne!(frame(Mood::Idle, ART, 3, 0), ART);
        assert_eq!(frame(Mood::Idle, ART, 4, 0), ART);
    }

    #[test]
    fn resting_half_closes_the_eyes() {
        assert!(!frame(Mood::Resting, ART, 0, 0).contains('o'));
        assert!(frame(Mood::Resting, ART, 0, 0).contains('.'));
    }

    #[test]
    fn petted_pulses_hearts() {
        assert!(frame(Mood::Petted, ART, 0, 0).contains("<3"));
        assert_ne!(frame(Mood::Petted, ART, 0, 0), frame(Mood::Petted, ART, 1, 0));
    }

    #[test]
    fn sleeping_closes_symbol_eyes_too() {
        let art = "( \u{25C9}   \u{25C9} )\n";
        let f = frame(Mood::Sleeping, art, 0, 0);
        assert!(!f.contains('\u{25C9}'), "symbol eyes must close when asleep");
    }

    #[test]
    fn working_spins() {
        assert_ne!(frame(Mood::Working, ART, 0, 0), frame(Mood::Working, ART, 1, 0));
    }

    #[test]
    fn no_glitch_below_the_threshold() {
        for tick in 0..200 {
            let f = frame(Mood::Idle, ART, tick, 40);
            assert_eq!(f, if tick % 4 == 3 { closed_eyes(ART) } else { ART.to_string() });
        }
    }

    #[test]
    fn high_chaos_throws_bursts_not_a_constant_glitch() {
        let mut corrupted = 0;
        let mut clean = 0;
        for tick in 0..200 {
            let f = frame(Mood::Idle, ART, tick, 100);
            if f.contains('#') || f.contains('%') || f.contains('?') {
                corrupted += 1;
            } else {
                clean += 1;
            }
        }
        assert!(corrupted > 20, "bursts should fire often at chaos 100: {}", corrupted);
        assert!(clean > 5, "glitching must not be constant at chaos 100: {}", clean);
    }

    #[test]
    fn glitch_never_touches_spaces_or_newlines() {
        let f = frame(Mood::Idle, ART, 7, 100);
        let mut glitched = false;
        for (a, b) in f.chars().zip(ART.chars()) {
            if b == ' ' || b == '\n' {
                assert_eq!(a, b, "whitespace must survive a glitch frame");
            } else if a != b {
                glitched = true;
            }
        }
        assert!(glitched);
    }

    #[test]
    fn severity_rises_with_chaos() {
        let count = |chaos: i64| -> usize {
            glitch_frame(ART, 42, chaos)
                .chars()
                .zip(ART.chars())
                .filter(|(a, b)| a != b && *b != ' ' && *b != '\n')
                .count()
        };
        assert!(count(90) > count(55));
    }

    #[test]
    fn frames_are_deterministic() {
        assert_eq!(frame(Mood::Idle, ART, 9, 80), frame(Mood::Idle, ART, 9, 80));
    }

    const EGG_ART: &str = "   o   o   \n  (o.o)   \n   ~~~~~   \n  <(*_*)>  \n";

    #[test]
    fn hatch_frames_hold_the_art_box_then_return_the_buddy() {
        // Every egg frame keeps the base art's shape so the popup never
        // resizes mid-animation, and the final step is the buddy itself.
        for step in 0..HATCH_TICKS {
            let f = hatch_frame(EGG_ART, step);
            assert_eq!(f.lines().count(), EGG_ART.lines().count(), "step {}", step);
            let w = f.lines().map(|l| l.chars().count()).max().unwrap();
            assert!(
                w <= EGG_ART.lines().map(|l| l.chars().count()).max().unwrap(),
                "step {}",
                step
            );
        }
        assert_eq!(hatch_frame(EGG_ART, HATCH_TICKS), EGG_ART);
        assert_eq!(hatch_frame(EGG_ART, 999), EGG_ART);
    }

    #[test]
    fn hatch_walks_egg_crack_then_emerge() {
        let intact = hatch_frame(EGG_ART, 0);
        assert!(!intact.contains('X'), "step 0 must be an intact egg");
        assert!(
            hatch_frame(EGG_ART, 0) != hatch_frame(EGG_ART, 1),
            "the egg should wobble"
        );
        let cracked = hatch_frame(EGG_ART, 5);
        assert!(cracked.contains('X'), "mid-hatch must show the crack");
        let open = hatch_frame(EGG_ART, 9);
        assert!(open.contains('o'), "emerge step must peek the eyes");
    }

    #[test]
    fn hatch_skips_an_art_box_too_small_for_an_egg() {
        let tiny = " o \n";
        assert_eq!(hatch_frame(tiny, 5), tiny, "a 1-line box cannot hold an egg");
    }

    #[test]
    fn hatch_fits_a_three_line_art() {
        let short = " (o.o)\n <(*_*)>\n  ~~~\n";
        assert_ne!(hatch_frame(short, 5), short, "a 3-line box still animates");
        let f = hatch_frame(short, 5);
        assert!(f.contains('X'), "3-line crack must show");
        assert_eq!(hatch_frame(short, HATCH_TICKS), short);
    }

    #[test]
    fn hatch_fits_a_small_art_box() {
        let small = " (o.o)\n <(*_*)>\n  ~~~~\n  ~~~~\n";
        let f = hatch_frame(small, 5);
        assert_eq!(f.lines().count(), small.lines().count());
        let max_w = small.lines().map(|l| l.chars().count()).max().unwrap();
        for l in f.lines() {
            assert!(l.chars().count() <= max_w, "overflow: {:?} > {}", l, max_w);
        }
    }

    #[test]
    fn chance_and_severity_bands() {
        assert_eq!(glitch_chance(49), 0);
        assert_eq!(glitch_chance(50), 10);
        assert_eq!(glitch_chance(100), 70);
        assert!(glitch_severity(55) < glitch_severity(95));
    }
}
