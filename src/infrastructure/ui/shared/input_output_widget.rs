use std::rc::{Rc, Weak};

use adw::prelude::*;
use gtk::glib;

use super::code_view::CodeView;
use crate::domain::error::DomainResult;
use crate::infrastructure::ui::feature_detail_page::toast;

pub struct Options {
    pub input_label: &'static str,
    pub output_label: &'static str,
    pub input_height: i32,
    pub output_height: i32,
    pub input_language: Option<&'static str>,
    pub output_language: Option<&'static str>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            input_label: "Input",
            output_label: "Output",
            input_height: 180,
            output_height: 180,
            input_language: None,
            output_language: None,
        }
    }
}

struct Inner {
    input_view: CodeView,
    output_view: CodeView,
    output_panel: gtk::Box,
    buttons_box: gtk::Box,
    extra_box: gtk::Box,
}

/// Action toolbar above an input editor and a read-only output, side by side
/// when there is room and stacked otherwise. Button callbacks hold a weak
/// handle, and the root widget owns the strong one, so closing the page frees
/// everything.
#[derive(Clone)]
pub struct InputOutputWidget {
    inner: Rc<Inner>,
    root: gtk::Box,
}

pub fn timestamp() -> String {
    glib::DateTime::now_local()
        .and_then(|d| d.format("%H:%M:%S"))
        .map(Into::into)
        .unwrap_or_default()
}

fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
    let btn = gtk::Button::builder()
        .icon_name(icon)
        .tooltip_text(tooltip)
        .valign(gtk::Align::Center)
        .build();
    btn.add_css_class("flat");
    btn
}

/// A rounded card with a header row (title + `actions`) above a scrolled editor.
fn panel(
    label: &str,
    view: &impl IsA<gtk::Widget>,
    min_height: i32,
    actions: &[gtk::Button],
) -> gtk::Box {
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    header.add_css_class("dt-panel-header");
    let lbl = gtk::Label::builder()
        .label(label)
        .xalign(0.0)
        .hexpand(true)
        .build();
    lbl.add_css_class("heading");
    header.append(&lbl);
    for action in actions {
        header.append(action);
    }

    let scroll = gtk::ScrolledWindow::builder()
        .min_content_height(min_height)
        .vexpand(true)
        .hexpand(true)
        .child(view)
        .build();

    let panel = gtk::Box::new(gtk::Orientation::Vertical, 0);
    panel.add_css_class("card");
    panel.add_css_class("dt-panel");
    panel.set_overflow(gtk::Overflow::Hidden);
    panel.append(&header);
    panel.append(&scroll);
    panel
}

impl InputOutputWidget {
    pub fn new(opts: Options) -> Self {
        let input_view = CodeView::new(true, opts.input_language);
        let output_view = CodeView::new(false, opts.output_language);

        let clear_btn = icon_button("edit-clear-all-symbolic", "Clear");
        let copy_btn = icon_button("edit-copy-symbolic", "Copy output");

        let input_panel = panel(
            opts.input_label,
            input_view.widget(),
            opts.input_height,
            std::slice::from_ref(&clear_btn),
        );
        let output_panel = panel(
            opts.output_label,
            output_view.widget(),
            opts.output_height,
            std::slice::from_ref(&copy_btn),
        );

        let panes = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        panes.set_homogeneous(true);
        panes.append(&input_panel);
        panes.append(&output_panel);

        let breakpoint = adw::Breakpoint::new(
            adw::BreakpointCondition::parse("max-width: 720px").expect("valid breakpoint"),
        );
        breakpoint.add_setter(
            &panes,
            "orientation",
            Some(&gtk::Orientation::Vertical.to_value()),
        );
        let panes_bin = adw::BreakpointBin::builder()
            .width_request(300)
            .height_request(360)
            .vexpand(true)
            .child(&panes)
            .build();
        panes_bin.add_breakpoint(breakpoint);

        let extra_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let toolbar = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        toolbar.append(&extra_box);
        toolbar.append(&buttons_box);

        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.set_vexpand(true);
        root.append(&toolbar);
        root.append(&panes_bin);

        let inner = Rc::new(Inner {
            input_view,
            output_view,
            output_panel,
            buttons_box,
            extra_box,
        });
        let owned = Rc::clone(&inner);
        root.connect_destroy(move |_| {
            let _ = &owned;
        });
        let this = Self { inner, root };

        let weak = this.weak();
        clear_btn.connect_clicked(move |_| {
            if let Some(tool) = Self::upgrade(&weak) {
                tool.clear();
            }
        });
        let weak = this.weak();
        copy_btn.connect_clicked(move |_| {
            if let Some(tool) = Self::upgrade(&weak) {
                tool.copy_output();
            }
        });
        this
    }

    fn weak(&self) -> (Weak<Inner>, glib::WeakRef<gtk::Box>) {
        (Rc::downgrade(&self.inner), self.root.downgrade())
    }

    fn upgrade((inner, root): &(Weak<Inner>, glib::WeakRef<gtk::Box>)) -> Option<Self> {
        Some(Self {
            inner: inner.upgrade()?,
            root: root.upgrade()?,
        })
    }

    pub fn add_button(&self, label: &str, callback: impl Fn(&Self) + 'static) -> gtk::Button {
        self.add_button_with_class(label, Some("suggested-action"), callback)
    }

    pub fn add_button_with_class(
        &self,
        label: &str,
        css_class: Option<&str>,
        callback: impl Fn(&Self) + 'static,
    ) -> gtk::Button {
        let btn = gtk::Button::with_label(label);
        if let Some(c) = css_class {
            btn.add_css_class(c);
        }
        let weak = self.weak();
        btn.connect_clicked(move |_| {
            if let Some(tool) = Self::upgrade(&weak) {
                callback(&tool);
            }
        });
        self.inner.buttons_box.append(&btn);
        btn
    }

    /// Adds a widget (option, selector…) to the toolbar, before the buttons.
    pub fn add_extra_widget(&self, widget: &impl IsA<gtk::Widget>) {
        self.inner.extra_box.append(widget);
    }

    pub fn input(&self) -> String {
        self.inner.input_view.text()
    }

    #[allow(dead_code)]
    pub fn set_input(&self, text: &str) {
        self.inner.input_view.set_text(text);
    }

    pub fn set_input_language(&self, language: Option<&str>) {
        self.inner.input_view.set_language(language);
    }

    pub fn set_output_language(&self, language: Option<&str>) {
        self.inner.output_view.set_language(language);
    }

    pub fn set_output(&self, text: &str) {
        self.inner
            .output_view
            .widget()
            .remove_css_class("dt-error-text");
        self.inner.output_panel.remove_css_class("dt-error-frame");
        self.inner.output_view.set_text(text);
    }

    pub fn set_error(&self, message: &str) {
        self.inner
            .output_view
            .widget()
            .add_css_class("dt-error-text");
        self.inner.output_panel.add_css_class("dt-error-frame");
        self.inner
            .output_view
            .set_text(&format!("⚠ Error: {message}"));
    }

    /// Shows a transform's outcome: the output, or the error message.
    pub fn report(&self, result: DomainResult<String>) {
        match result {
            Ok(out) => self.set_output(&out),
            Err(e) => self.set_error(e.message()),
        }
    }

    fn copy_output(&self) {
        let text = self.inner.output_view.text();
        if text.is_empty() {
            return;
        }
        self.root.clipboard().set_text(&text);
        toast(&self.root, "Copied to clipboard");
    }

    fn clear(&self) {
        self.inner.input_view.set_text("");
        self.set_output("");
    }

    pub fn into_widget(self) -> gtk::Widget {
        self.root.upcast()
    }
}
