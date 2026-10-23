//! Scripted walk-through of the app, run by `update_screenshots.sh` to take
//! the README screenshots and record the demo GIF.
//!
//! Enabled by `DUPOT_DEMO_DIR`. The app talks to the script on stdout:
//! `DUPOT_SHOT <name>` asks for a screenshot, then the app waits until
//! `<dir>/<name>.png` exists; `DUPOT_RECORD start|stop` brackets the GIF;
//! `DUPOT_DONE` ends the run.

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use adw::prelude::*;
use gtk::glib;

const DIR_ENV: &str = "DUPOT_DEMO_DIR";
/// SQLite database the SQL viewer part connects to.
const DB_ENV: &str = "DUPOT_DEMO_DB";

fn dir() -> Option<PathBuf> {
    std::env::var_os(DIR_ENV).map(PathBuf::from)
}

pub fn active() -> bool {
    dir().is_some()
}

pub fn database() -> Option<String> {
    std::env::var(DB_ENV).ok()
}

pub async fn pause(ms: u64) {
    glib::timeout_future(Duration::from_millis(ms)).await;
}

fn say(line: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// Lets the UI settle, then waits for the script to capture the screen.
pub async fn shot(name: &str) {
    let Some(dir) = dir() else { return };
    pause(700).await;
    let file = dir.join(format!("{name}.png"));
    say(&format!("DUPOT_SHOT {name}"));
    let start = Instant::now();
    while !file.exists() && start.elapsed() < Duration::from_secs(10) {
        pause(100).await;
    }
}

pub async fn record(on: bool) {
    say(if on {
        "DUPOT_RECORD start"
    } else {
        "DUPOT_RECORD stop"
    });
    // The recorder takes a moment to start, and to catch the last frames.
    pause(1000).await;
}

pub fn done() {
    say("DUPOT_DONE");
}

/// Types `text` one character at a time, as a person would.
pub async fn type_into(editable: &impl IsA<gtk::Editable>, text: &str) {
    editable.set_text("");
    for c in text.chars() {
        let position = editable.text().chars().count() as i32;
        editable.insert_text(&c.to_string(), &mut position.clone());
        pause(45).await;
    }
}

pub async fn type_into_buffer(buffer: &gtk::TextBuffer, text: &str) {
    for c in text.chars() {
        buffer.insert_at_cursor(&c.to_string());
        pause(35).await;
    }
}

/// First widget of type `W` under `root` (depth first) matching `pred`.
pub fn find<W: IsA<gtk::Widget>>(
    root: &impl IsA<gtk::Widget>,
    pred: &impl Fn(&W) -> bool,
) -> Option<W> {
    let mut child = root.as_ref().first_child();
    while let Some(widget) = child {
        if let Some(w) = widget.downcast_ref::<W>()
            && pred(w)
        {
            return Some(w.clone());
        }
        if let Some(found) = find(&widget, pred) {
            return Some(found);
        }
        child = widget.next_sibling();
    }
    None
}
