use std::rc::Rc;

use chrono::Utc;
use gtk::prelude::*;

use crate::domain::feature::encoding::jwt_sign::{JwtAlgorithm, SAMPLE_PAYLOAD, sign_or_verify};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

struct Controls {
    algorithm: gtk::DropDown,
    key: gtk::TextView,
}

impl Controls {
    fn key(&self) -> String {
        let buffer = self.key.buffer();
        let (start, end) = buffer.bounds();
        buffer.text(&start, &end, false).to_string()
    }

    fn update(&self, t: &InputOutputWidget) {
        let input = t.input();
        if input.trim().is_empty() {
            t.set_output("");
            return;
        }
        let algorithm = JwtAlgorithm::ALL[self.algorithm.selected() as usize];
        t.report(sign_or_verify(
            &input,
            algorithm,
            self.key().trim(),
            Utc::now(),
        ));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Payload (JSON) to sign, or token to verify",
            output_label: "Result",
            input_language: Some("json"),
            ..Default::default()
        });

        let labels: Vec<&str> = JwtAlgorithm::ALL.iter().map(|a| a.label()).collect();
        let controls = Rc::new(Controls {
            algorithm: gtk::DropDown::from_strings(&labels),
            key: gtk::TextView::builder()
                .monospace(true)
                .wrap_mode(gtk::WrapMode::Char)
                .top_margin(6)
                .bottom_margin(6)
                .left_margin(8)
                .right_margin(8)
                .build(),
        });
        controls.algorithm.set_tooltip_text(Some(
            "Signing algorithm (verification reads it from the token)",
        ));
        controls.key.buffer().set_text("your-256-bit-secret");

        let algo_lbl = gtk::Label::new(Some("Algorithm"));
        algo_lbl.add_css_class("dim-label");
        let key_lbl = gtk::Label::builder()
            .label("Secret (HS*) or PEM key: private to sign, public or private to verify")
            .xalign(0.0)
            .build();
        key_lbl.add_css_class("dim-label");
        let key_scroll = gtk::ScrolledWindow::builder()
            .min_content_height(72)
            .max_content_height(160)
            .propagate_natural_height(true)
            .hexpand(true)
            .child(&controls.key)
            .build();
        key_scroll.add_css_class("card");
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.append(&algo_lbl);
        row.append(&controls.algorithm);
        row.append(&key_lbl);
        let rows = gtk::Box::new(gtk::Orientation::Vertical, 8);
        rows.set_hexpand(true);
        rows.append(&row);
        rows.append(&key_scroll);
        tool.add_extra_widget(&rows);

        let refresh = {
            let (controls, tool) = (Rc::downgrade(&controls), tool.downgrade());
            move || {
                if let (Some(c), Some(t)) = (controls.upgrade(), tool.upgrade()) {
                    c.update(&t);
                }
            }
        };
        let r = refresh.clone();
        controls.algorithm.connect_selected_notify(move |_| r());
        controls.key.buffer().connect_changed(move |_| refresh());
        let c = Rc::clone(&controls);
        tool.connect_input_changed(move |t| c.update(t));
        tool.set_input(SAMPLE_PAYLOAD);
        tool.into_widget()
    }
}
