use std::rc::Rc;

use gtk::prelude::*;

use crate::domain::feature::string::regex_tester::{RegexOptions, find_matches, replace, report};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

struct Controls {
    pattern: gtk::Entry,
    replacement: gtk::Entry,
    case_insensitive: gtk::CheckButton,
    multiline: gtk::CheckButton,
    dot_all: gtk::CheckButton,
}

impl Controls {
    fn options(&self) -> RegexOptions {
        RegexOptions {
            case_insensitive: self.case_insensitive.is_active(),
            multiline: self.multiline.is_active(),
            dot_matches_newline: self.dot_all.is_active(),
        }
    }

    fn update(&self, t: &InputOutputWidget) {
        let (pattern, text) = (self.pattern.text(), t.input());
        if pattern.is_empty() {
            t.highlight_input(&[]);
            t.set_output("");
            return;
        }
        match find_matches(&pattern, self.options(), &text) {
            Ok(matches) => {
                let ranges: Vec<_> = matches.iter().map(|m| (m.start, m.end)).collect();
                t.highlight_input(&ranges);
                let replacement = self.replacement.text();
                if replacement.is_empty() {
                    t.set_output(&report(&matches));
                } else {
                    t.report(replace(&pattern, self.options(), &text, &replacement));
                }
            }
            Err(e) => {
                t.highlight_input(&[]);
                t.set_error(e.message());
            }
        }
    }
}

fn entry(placeholder: &str) -> gtk::Entry {
    let entry = gtk::Entry::builder()
        .placeholder_text(placeholder)
        .hexpand(true)
        .build();
    entry.add_css_class("monospace");
    entry
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Test text",
            output_label: "Matches",
            ..Default::default()
        });

        let controls = Rc::new(Controls {
            pattern: entry("Regular expression, e.g. (\\w+)@(\\w+)\\.com"),
            replacement: entry("Replacement (optional): $1, ${name}…"),
            case_insensitive: gtk::CheckButton::with_label("Ignore case"),
            multiline: gtk::CheckButton::with_label("Multiline"),
            dot_all: gtk::CheckButton::with_label("Dot matches newline"),
        });

        let row1 = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row1.append(&controls.pattern);
        row1.append(&controls.case_insensitive);
        row1.append(&controls.multiline);
        row1.append(&controls.dot_all);
        let rows = gtk::Box::new(gtk::Orientation::Vertical, 8);
        rows.set_hexpand(true);
        rows.append(&row1);
        rows.append(&controls.replacement);
        tool.add_extra_widget(&rows);

        let refresh = {
            let (controls, tool) = (Rc::downgrade(&controls), tool.downgrade());
            move || {
                if let (Some(controls), Some(tool)) = (controls.upgrade(), tool.upgrade()) {
                    controls.update(&tool);
                }
            }
        };
        let r = refresh.clone();
        controls.pattern.connect_changed(move |_| r());
        let r = refresh.clone();
        controls.replacement.connect_changed(move |_| r());
        for check in [
            &controls.case_insensitive,
            &controls.multiline,
            &controls.dot_all,
        ] {
            let r = refresh.clone();
            check.connect_toggled(move |_| r());
        }
        let c = Rc::clone(&controls);
        tool.connect_input_changed(move |t| c.update(t));
        tool.into_widget()
    }
}
