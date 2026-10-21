#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod domain;
mod infrastructure;

use adw::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let app = infrastructure::ui::app_window::build_application();
    app.run()
}
