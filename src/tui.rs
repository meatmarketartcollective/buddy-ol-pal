use crate::anim;
use crate::species::{self, Species};
use crate::state::{self, CompanionState, Event, Mood};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

fn last_line(last: Option<&Event>, now: i64) -> String {
    match last {
        None => "nothing yet. try: buddy run -- <cmd>".to_string(),
        Some(ev) => format!(
            "last: {}, {}s ago",
            ev.kind,
            (now - ev.ts).max(0)
        ),
    }
}

fn draw(
    f: &mut Frame,
    st: &CompanionState,
    sp: &Species,
    mood: Mood,
    last: Option<&Event>,
    now: i64,
    tick: u64,
    chaos: i64,
    hatch: Option<(u64, &str)>,
) {
    let area = f.area();
    let base_art = sp.art.frame_for(mood, tick);
    let (art, blurb) = match hatch {
        Some((start, msg)) if tick.saturating_sub(start) < anim::HATCH_TICKS => (
            anim::hatch_frame(base_art, tick.saturating_sub(start)),
            msg.to_string(),
        ),
        _ => (
            anim::frame(mood, base_art, tick, chaos),
            species::pick_line(sp.lines.for_mood(mood), now).to_string(),
        ),
    };
    let mood_style = match mood {
        Mood::Idle => Style::default().fg(Color::Gray),
        Mood::Working => Style::default().fg(Color::Yellow),
        Mood::Celebrate => Style::default().fg(Color::Green),
        Mood::Miffed => Style::default().fg(Color::Red),
        Mood::Sleeping => Style::default().fg(Color::DarkGray),
        Mood::Resting => Style::default().fg(Color::Gray),
        Mood::Petted => Style::default().fg(Color::Magenta),
    };
    let art_style = match mood {
        Mood::Idle | Mood::Resting => Style::default().fg(Color::White),
        _ => mood_style,
    };
    let mut lines: Vec<Line> = Vec::new();
    for l in art.lines() {
        lines.push(Line::from(Span::styled(l.to_string(), art_style)));
    }
    lines.push(Line::from(Span::styled(
        format!(
            "{} the {} [{}] L{} ({})",
            st.name, sp.name, sp.tier, st.level, mood.label()
        ),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(blurb.to_string(), mood_style)));
    lines.push(Line::from(Span::styled(
        last_line(last, now),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(Span::styled(
        species::speak(sp, mood, (now / 5) as u64).to_string(),
        Style::default().fg(Color::Cyan),
    )));

    let w = 36u16;
    let h = (art.lines().count() as u16).saturating_add(7);
    let popup = Rect {
        x: area.width.saturating_sub(w + 1),
        y: area.height.saturating_sub(h + 1),
        width: w.min(area.width),
        height: h.min(area.height),
    };
    let para = Paragraph::new(lines)
        .block(Block::bordered().title("/buddy ol pal"))
        .alignment(Alignment::Left);
    f.render_widget(para, popup);
}

pub fn run_ui() -> i32 {
    let dir = state::config_dir();
    let mut st = state::load_or_hatch(&dir);
    if !st.enabled {
        println!("the buddy is off (buddy on to wake it)");
        return 0;
    }
    let sp = species::resolve(&st.species);

    let mut stdout = io::stdout();
    if crossterm::terminal::enable_raw_mode().is_err() {
        eprintln!("buddy: needs a real terminal (tty)");
        return 1;
    }
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).ok();
    let mut terminal = match Terminal::new(CrosstermBackend::new(stdout)) {
        Ok(t) => t,
        Err(_) => {
            crossterm::terminal::disable_raw_mode().ok();
            return 1;
        }
    };

    let mut quit = false;
    let mut tick: u64 = 0;
    let mut wake: Option<i64> = None;
    // A hatch or rehatch that just happened (fresh egg, or the species and
    // ascension count another process just changed) plays the egg animation
    // once, then the normal mood art takes back over.
    let mut seen_hatched_at = st.hatched_at;
    let mut seen_ascensions = st.ascensions;
    let mut seen_species = st.species.clone();
    let mut fresh_at_start = st.hatched_at + 2 >= state::now_secs();
    let mut hatch_start: Option<u64> = None;
    let mut hatch_msg = String::new();
    loop {
        while crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(crossterm::event::Event::Key(k)) = crossterm::event::read() {
                if k.kind == crossterm::event::KeyEventKind::Press {
                    if matches!(
                        k.code,
                        crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc
                    ) {
                        quit = true;
                    } else {
                        // Any other keypress wakes the buddy from sleep.
                        wake = Some(state::now_secs());
                    }
                }
            }
        }
        if quit {
            break;
        }

        let now = state::now_secs();
        let events = state::read_events_tail(&dir, 8);
        let active = state::is_active(&dir);
        // Pick up changes another process made to the state file (a feed or
        // ascension in a sibling pane) so the UI stays live with it.
        if let Ok(text) = std::fs::read_to_string(dir.join("state.toml")) {
            if let Some(fresh) = state::parse_state(&text) {
                if fresh.hatched_at != seen_hatched_at || fresh_at_start {
                    seen_hatched_at = fresh.hatched_at;
                    fresh_at_start = false;
                    hatch_start = Some(tick);
                    hatch_msg = format!(
                        "It hatched! {} the {} ({})",
                        fresh.name,
                        species::display_name(&fresh.species),
                        species::get(&fresh.species)
                            .unwrap_or_else(species::fallback)
                            .tier
                    );
                } else if fresh.ascensions > seen_ascensions || fresh.species != seen_species {
                    seen_ascensions = fresh.ascensions;
                    seen_species = fresh.species.clone();
                    hatch_start = Some(tick);
                    hatch_msg = format!(
                        "Rehatched! {} the {} ({})",
                        fresh.name,
                        species::display_name(&fresh.species),
                        species::get(&fresh.species)
                            .unwrap_or_else(species::fallback)
                            .tier
                    );
                }
                st = fresh;
            }
        }
        let mut mood = state::current_mood(active, &events, now);
        if mood == Mood::Sleeping {
            if let Some(w) = wake {
                if now - w <= state::REST_SECS {
                    mood = Mood::Resting;
                }
            }
        }
        st.mood = mood.label().to_string();
        let chaos = state::effective_chaos(&st, &events, now);

        let hatch = hatch_start.map(|s| (s, hatch_msg.as_str()));
        let _ = terminal.draw(|f| {
            draw(f, &st, sp, mood, events.last(), now, tick, chaos, hatch)
        });
        tick += 1;
        std::thread::sleep(Duration::from_millis(250));
    }

    let _ = state::save_state(&dir, &st);
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )
    .ok();
    crossterm::terminal::disable_raw_mode().ok();
    terminal.show_cursor().ok();
    0
}

#[cfg(test)]
mod tests {
    use super::{draw, last_line};
    use crate::species;
    use crate::state::{CompanionState, Event, Mood};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn fixture() -> (CompanionState, &'static species::Species) {
        let sp = species::fallback();
        let st = CompanionState {
            version: 6,
            name: "Fig".to_string(),
            species: sp.name.to_string(),
            hatched_at: 0,
            mood: "idle".to_string(),
            xp: 3,
            level: 1,
            discovered: vec![],
            chaos: 0,
            enabled: true,
            xp_today: 0,
            xp_day: 0,
            ascensions: 0,
        };
        (st, sp)
    }

    fn buffer_text(b: &ratatui::buffer::Buffer) -> String {
        b.content.iter().map(|c| c.symbol().to_string()).collect()
    }

    #[test]
    fn no_events_hint_the_first_run() {
        assert_eq!(
            last_line(None, 100),
            "nothing yet. try: buddy run -- <cmd>"
        );
    }

    #[test]
    fn last_event_shows_kind_and_age() {
        let ev = Event {
            ts: 93,
            kind: "command_failed".to_string(),
            duration_secs: 2,
            exit_code: 1,
        };
        assert_eq!(last_line(Some(&ev), 100), "last: command_failed, 7s ago");
    }

    #[test]
    fn clock_skew_never_shows_negative_age() {
        let ev = Event {
            ts: 500,
            kind: "fed".to_string(),
            duration_secs: 0,
            exit_code: 0,
        };
        assert_eq!(last_line(Some(&ev), 100), "last: fed, 0s ago");
    }

    #[test]
    fn ui_draws_title_name_and_hint() {
        let (st, sp) = fixture();
        let ev = Event {
            ts: 98,
            kind: "build_finished".to_string(),
            duration_secs: 42,
            exit_code: 0,
        };
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|f| draw(f, &st, sp, Mood::Idle, Some(&ev), 100, 0, 0, None))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("/buddy ol pal"), "box title missing");
        assert!(text.contains("Fig the "), "pet line missing");
        assert!(text.contains("L1"), "level missing");
        assert!(
            text.contains(&format!("[{}]", sp.tier)),
            "rarity tier missing from the header: {}",
            sp.tier
        );
        assert!(text.contains("last: build_finished, 2s ago"), "last line missing");
    }

    #[test]
    fn empty_history_draws_the_first_run_hint() {
        let (st, sp) = fixture();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|f| draw(f, &st, sp, Mood::Idle, None, 100, 0, 0, None))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("nothing yet"));
    }

    #[test]
    fn mood_sets_the_status_word() {
        let (st, sp) = fixture();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|f| draw(f, &st, sp, Mood::Miffed, None, 100, 0, 0, None))
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("(miffed)"), "mood label missing from the pet line");
    }

    #[test]
    fn hatch_frames_take_over_until_the_animation_ends() {
        let (st, sp) = fixture();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|f| {
                draw(
                    f,
                    &st,
                    sp,
                    Mood::Idle,
                    None,
                    100,
                    0,
                    0,
                    Some((0, "It hatched! Fig the ember (legendary)")),
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("It hatched!"), "hatch blurb missing");
        // A tick past the animation length, the normal art and blurb return.
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal
            .draw(|f| {
                draw(
                    f,
                    &st,
                    sp,
                    Mood::Idle,
                    None,
                    100,
                    crate::anim::HATCH_TICKS,
                    0,
                    Some((0, "It hatched! Fig the ember (legendary)")),
                )
            })
            .unwrap();
        let text = buffer_text(terminal.backend().buffer());
        assert!(!text.contains("It hatched!"), "hatch blurb must end");
    }

    #[test]
    fn ui_survives_a_tiny_terminal() {
        let (st, sp) = fixture();
        let ev = Event {
            ts: 98,
            kind: "fed".to_string(),
            duration_secs: 0,
            exit_code: 0,
        };
        let mut terminal = Terminal::new(TestBackend::new(20, 5)).unwrap();
        terminal
            .draw(|f| draw(f, &st, sp, Mood::Idle, Some(&ev), 100, 0, 0, None))
            .unwrap();
    }
}
