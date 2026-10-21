use std::rc::Rc;

use adw::prelude::*;

use crate::domain::feature::string::text_diff::{DiffOptions, diff};
use crate::infrastructure::ui::feature_contract::FeatureContract;
use crate::infrastructure::ui::feature_detail_page::toast;
use crate::infrastructure::ui::shared::code_view::CodeView;
use crate::infrastructure::ui::shared::input_output_widget::{icon_button, panel};

pub struct Feature;

struct DiffView {
    original: CodeView,
    modified: CodeView,
    output: CodeView,
    summary: gtk::Label,
    ignore_whitespace: gtk::CheckButton,
    ignore_case: gtk::CheckButton,
    context: gtk::SpinButton,
}

impl DiffView {
    fn update(&self) {
        let (original, modified) = (self.original.text(), self.modified.text());
        if original.is_empty() && modified.is_empty() {
            self.output.set_text("");
            self.summary.set_text("");
            return;
        }
        let options = DiffOptions {
            ignore_whitespace: self.ignore_whitespace.is_active(),
            ignore_case: self.ignore_case.is_active(),
            context_lines: self.context.value() as usize,
        };
        let (unified, stats) = diff(&original, &modified, options);
        self.output.set_text(&unified);
        self.summary.set_text(&stats.summary());
    }
}

impl FeatureContract for Feature {
    fn widget(&self) -> gtk::Widget {
        let view = Rc::new(DiffView {
            original: CodeView::new(true, None),
            modified: CodeView::new(true, None),
            output: CodeView::new(false, Some("diff")),
            summary: gtk::Label::builder().xalign(0.0).hexpand(true).build(),
            ignore_whitespace: gtk::CheckButton::with_label("Ignore whitespace"),
            ignore_case: gtk::CheckButton::with_label("Ignore case"),
            context: gtk::SpinButton::with_range(0.0, 100.0, 1.0),
        });
        view.context.set_value(3.0);
        view.context.set_tooltip_text(Some("Context lines"));
        view.summary.add_css_class("dim-label");

        let inputs = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        inputs.set_homogeneous(true);
        inputs.append(&panel("Original", view.original.widget(), 180, &[]));
        inputs.append(&panel("Modified", view.modified.widget(), 180, &[]));
        let breakpoint = adw::Breakpoint::new(
            adw::BreakpointCondition::parse("max-width: 720px").expect("valid breakpoint"),
        );
        breakpoint.add_setter(
            &inputs,
            "orientation",
            Some(&gtk::Orientation::Vertical.to_value()),
        );
        let inputs_bin = adw::BreakpointBin::builder()
            .width_request(300)
            .height_request(200)
            .child(&inputs)
            .build();
        inputs_bin.add_breakpoint(breakpoint);

        let context_lbl = gtk::Label::new(Some("Context"));
        context_lbl.add_css_class("dim-label");
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        toolbar.append(&view.summary);
        toolbar.append(&view.ignore_whitespace);
        toolbar.append(&view.ignore_case);
        toolbar.append(&context_lbl);
        toolbar.append(&view.context);

        let copy_btn = icon_button("edit-copy-symbolic", "Copy diff");
        let output_panel = panel(
            "Unified diff",
            view.output.widget(),
            200,
            std::slice::from_ref(&copy_btn),
        );

        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.set_vexpand(true);
        root.append(&inputs_bin);
        root.append(&toolbar);
        root.append(&output_panel);

        let weak = Rc::downgrade(&view);
        let refresh = move || {
            if let Some(view) = weak.upgrade() {
                view.update();
            }
        };
        for code in [&view.original, &view.modified] {
            let r = refresh.clone();
            code.connect_changed(r);
        }
        for check in [&view.ignore_whitespace, &view.ignore_case] {
            let r = refresh.clone();
            check.connect_toggled(move |_| r());
        }
        let r = refresh.clone();
        view.context.connect_value_changed(move |_| r());

        let weak = Rc::downgrade(&view);
        copy_btn.connect_clicked(move |btn| {
            if let Some(view) = weak.upgrade() {
                let text = view.output.text();
                if !text.is_empty() {
                    btn.clipboard().set_text(&text);
                    toast(btn, "Copied to clipboard");
                }
            }
        });

        // The page owns the view state; signal handlers only hold weak refs.
        root.connect_destroy(move |_| {
            let _ = &view;
        });
        root.upcast()
    }
}
