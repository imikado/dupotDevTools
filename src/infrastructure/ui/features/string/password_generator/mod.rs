use gtk::prelude::*;

use crate::domain::feature::string::password_generator::{
    MAX_COUNT, MAX_LENGTH, PasswordOptions, generate, inspect, strength,
};
use crate::domain::shared::format::number;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Check a password (stays on this machine)",
            output_label: "Passwords",
            ..Default::default()
        });

        let defaults = PasswordOptions::default();
        let length = gtk::SpinButton::with_range(4.0, MAX_LENGTH as f64, 1.0);
        length.set_value(defaults.length as f64);
        length.set_tooltip_text(Some("Length"));
        let count = gtk::SpinButton::with_range(1.0, MAX_COUNT as f64, 1.0);
        count.set_tooltip_text(Some("How many"));
        let check = |label: &str, active: bool| {
            let c = gtk::CheckButton::with_label(label);
            c.set_active(active);
            c
        };
        let lowercase = check("a-z", defaults.lowercase);
        let uppercase = check("A-Z", defaults.uppercase);
        let digits = check("0-9", defaults.digits);
        let symbols = check("!@#", defaults.symbols);
        let no_ambiguous = check("No look-alikes", defaults.exclude_ambiguous);
        no_ambiguous.set_tooltip_text(Some("Exclude I l 1 | O 0 o and quotes"));
        let entropy = gtk::Label::new(None);
        entropy.add_css_class("dim-label");

        let length_lbl = gtk::Label::new(Some("Length"));
        length_lbl.add_css_class("dim-label");
        tool.add_extra_widget(&length_lbl);
        tool.add_extra_widget(&length);
        tool.add_extra_widget(&count);
        for c in [&lowercase, &uppercase, &digits, &symbols, &no_ambiguous] {
            tool.add_extra_widget(c);
        }
        tool.add_extra_widget(&entropy);

        let generate_btn = tool.add_button("Generate", move |t| {
            let options = PasswordOptions {
                length: length.value() as usize,
                lowercase: lowercase.is_active(),
                uppercase: uppercase.is_active(),
                digits: digits.is_active(),
                symbols: symbols.is_active(),
                exclude_ambiguous: no_ambiguous.is_active(),
            };
            let bits = options.entropy_bits();
            entropy.set_text(&format!("≈ {} bits · {}", number(bits, 0), strength(bits)));
            t.report(generate(options, count.value() as usize));
        });
        tool.connect_input_changed(|t| {
            if t.input().is_empty() {
                t.set_output("");
            } else {
                t.set_output(&inspect(&t.input()));
            }
        });
        generate_btn.emit_clicked();
        tool.into_widget()
    }
}
