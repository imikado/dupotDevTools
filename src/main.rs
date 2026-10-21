#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use adw::prelude::*;
use dupot_dev_tools::infrastructure::ui::app_window;

fn main() -> gtk::glib::ExitCode {
    app_window::build_application().run()
}
