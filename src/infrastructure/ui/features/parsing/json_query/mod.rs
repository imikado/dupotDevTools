use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::domain::feature::parsing::json_query::{QueryLanguage, SAMPLE, run};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::shared::input_output_widget::{InputOutputWidget, Options};

pub struct Feature;

struct Controls {
    language: gtk::DropDown,
    expression: gtk::Entry,
    /// Bumped on every edit, so that a slow query's late result is dropped.
    generation: Cell<u64>,
}

impl Controls {
    fn language(&self) -> QueryLanguage {
        QueryLanguage::ALL[self.language.selected() as usize]
    }

    /// Runs off the main thread: a jq filter may loop forever.
    fn update(self: &Rc<Self>, t: &InputOutputWidget) {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let (expr, json) = (self.expression.text().to_string(), t.input());
        if expr.trim().is_empty() || json.trim().is_empty() {
            t.set_output("");
            return;
        }
        let language = self.language();
        let (controls, tool) = (Rc::downgrade(self), t.downgrade());
        glib::spawn_future_local(async move {
            let Ok(result) = gio::spawn_blocking(move || run(language, &expr, &json)).await else {
                return;
            };
            if let (Some(c), Some(t)) = (controls.upgrade(), tool.upgrade())
                && c.generation.get() == generation
            {
                t.report(result);
            }
        });
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let tool = InputOutputWidget::new(Options {
            input_label: "JSON",
            output_label: "Result",
            input_language: Some("json"),
            output_language: Some("json"),
            input_height: 260,
            output_height: 260,
        });

        let labels: Vec<&str> = QueryLanguage::ALL.iter().map(|l| l.label()).collect();
        let controls = Rc::new(Controls {
            language: gtk::DropDown::from_strings(&labels),
            expression: gtk::Entry::builder().hexpand(true).build(),
            generation: Cell::new(0),
        });
        controls.expression.add_css_class("monospace");
        controls
            .expression
            .set_placeholder_text(Some(QueryLanguage::JsonPath.placeholder()));
        tool.add_extra_widget(&controls.language);
        tool.add_extra_widget(&controls.expression);

        let refresh = {
            let (controls, tool) = (Rc::downgrade(&controls), tool.downgrade());
            move || {
                if let (Some(c), Some(t)) = (controls.upgrade(), tool.upgrade()) {
                    c.update(&t);
                }
            }
        };
        let r = refresh.clone();
        controls.expression.connect_changed(move |_| r());
        let weak_controls = Rc::downgrade(&controls);
        controls.language.connect_selected_notify(move |_| {
            if let Some(c) = weak_controls.upgrade() {
                c.expression
                    .set_placeholder_text(Some(c.language().placeholder()));
            }
            refresh();
        });
        let c = Rc::clone(&controls);
        tool.connect_input_changed(move |t| c.update(t));
        tool.set_input(SAMPLE);
        controls
            .expression
            .set_text("$.store.book[?@.price < 10].title");
        tool.into_widget()
    }
}
