use std::rc::Rc;

use gtk::prelude::*;

use crate::domain::feature::convert::color;
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

struct Controls {
    picker: gtk::ColorDialogButton,
    background: gtk::Entry,
}

impl Controls {
    fn update(&self, t: &InputOutputWidget) {
        let input = t.input();
        if input.trim().is_empty() {
            t.set_output("");
            return;
        }
        if let Ok(c) = color::parse(&input) {
            let rgba = gtk::gdk::RGBA::new(c.r as f32, c.g as f32, c.b as f32, c.alpha as f32);
            if self.picker.rgba() != rgba {
                self.picker.set_rgba(&rgba);
            }
        }
        t.report(color::describe(&input, Some(&self.background.text())));
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "Color (HEX, rgb(), hsl(), oklch(), oklab())",
            output_label: "Conversions",
            ..Default::default()
        });

        let controls = Rc::new(Controls {
            picker: gtk::ColorDialogButton::new(Some(gtk::ColorDialog::new())),
            background: gtk::Entry::builder()
                .placeholder_text("Contrast against… (optional)")
                .hexpand(true)
                .build(),
        });
        controls.picker.set_tooltip_text(Some("Pick a color"));
        controls.background.add_css_class("monospace");
        tool.add_extra_widget(&controls.picker);
        tool.add_extra_widget(&controls.background);

        let weak_tool = tool.downgrade();
        controls.picker.connect_rgba_notify(move |picker| {
            let Some(tool) = weak_tool.upgrade() else {
                return;
            };
            let rgba = picker.rgba();
            let picked =
                color::Color::rgb(rgba.red().into(), rgba.green().into(), rgba.blue().into());
            let current = color::parse(&tool.input()).map(|c| c.hex()).ok();
            if current.as_deref() != Some(&picked.hex()) {
                tool.set_input(&picked.hex());
            }
        });
        let (weak_controls, weak_tool) = (Rc::downgrade(&controls), tool.downgrade());
        controls.background.connect_changed(move |_| {
            if let (Some(c), Some(t)) = (weak_controls.upgrade(), weak_tool.upgrade()) {
                c.update(&t);
            }
        });
        let c = Rc::clone(&controls);
        tool.connect_input_changed(move |t| c.update(t));
        tool.set_input("#3584E4");
        tool.into_widget()
    }
}
