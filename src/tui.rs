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
        None => "nothing yet. try: tb run -- <cmd>".to_string(),
        Some(ev) => format!(
            "last: {}, {}s ago",
            ev.kind,
            (now - ev.ts).max(0)
        ),
    }
}

fn draw(f: &mut Frame, st: &CompanionState, sp: &Species, mood: Mood, last: Option<&Event>, now: i64) {
    let area = f.area();
    let art = sp.art.for_mood(mood);
    let blurb = species::pick_line(sp.lines.for_mood(mood), now);
    let mood_style = match mood {
        Mood::Idle => Style::default().fg(Color::Gray),
        Mood::Working => Style::default().fg(Color::Yellow),
        Mood::Celebrate => Style::default().fg(Color::Green),
        Mood::Miffed => Style::default().fg(Color::Red),
    };
    let mut lines: Vec<Line> = Vec::new();
    for l in art.lines() {
        lines.push(Line::from(Span::styled(
            l.to_string(),
            Style::default().fg(Color::White),
        )));
    }
    lines.push(Line::from(Span::styled(
        format!("{} the {} L{} ({})", st.name, sp.name, st.level, mood.label()),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(blurb.to_string(), mood_style)));
    lines.push(Line::from(Span::styled(
        last_line(last, now),
        Style::default().fg(Color::DarkGray),
    )));

    let w = 36u16;
    let h = 9u16;
    let popup = Rect {
        x: area.width.saturating_sub(w + 1),
        y: area.height.saturating_sub(h + 1),
        width: w.min(area.width),
        height: h.min(area.height),
    };
    let para = Paragraph::new(lines)
        .block(Block::bordered().title("hatch"))
        .alignment(Alignment::Left);
    f.render_widget(para, popup);
}

pub fn run_ui() -> i32 {
    let dir = state::config_dir();
    let mut st = state::load_or_hatch(&dir);
    let sp = species::resolve(&st.species);

    let mut stdout = io::stdout();
    if crossterm::terminal::enable_raw_mode().is_err() {
        eprintln!("hatch: needs a real terminal (tty)");
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
    loop {
        while crossterm::event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(crossterm::event::Event::Key(k)) = crossterm::event::read() {
                if k.kind == crossterm::event::KeyEventKind::Press
                    && matches!(
                        k.code,
                        crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc
                    )
                {
                    quit = true;
                }
            }
        }
        if quit {
            break;
        }

        let now = state::now_secs();
        let events = state::read_events_tail(&dir, 8);
        let active = state::is_active(&dir);
        let mood = state::current_mood(active, &events, now);
        st.mood = mood.label().to_string();

        let _ = terminal.draw(|f| draw(f, &st, sp, mood, events.last(), now));
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
