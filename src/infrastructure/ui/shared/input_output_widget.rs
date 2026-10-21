use std::rc::{Rc, Weak};

use gtk::glib;
use gtk::prelude::*;

use super::code_view::CodeView;

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
            input_height: 150,
            output_height: 250,
            input_language: None,
            output_language: None,
        }
    }
}

struct Inner {
    input_view: CodeView,
    output_view: CodeView,
    output_scroll: gtk::ScrolledWindow,
    buttons_box: gtk::Box,
    extra_box: gtk::Box,
    status_label: gtk::Label,
}

/// Input editor + action buttons + read-only output, the layout shared by
/// most single-transform tools. Button callbacks hold a weak handle, and the
/// root widget owns the strong one, so closing the page frees everything.
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

fn make_section(
    label: &str,
    view: &impl IsA<gtk::Widget>,
    height: i32,
) -> (gtk::Box, gtk::ScrolledWindow) {
    let section = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let lbl = gtk::Label::builder().label(label).xalign(0.0).build();
    lbl.add_css_class("heading");
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_size_request(-1, height);
    scroll.set_child(Some(view));
    scroll.add_css_class("card");
    section.append(&lbl);
    section.append(&scroll);
    (section, scroll)
}

impl InputOutputWidget {
    pub fn new(opts: Options) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        let input_view = CodeView::new(true, opts.input_language);
        let output_view = CodeView::new(false, opts.output_language);

        let buttons_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        buttons_box.set_halign(gtk::Align::Center);

        let status_label = gtk::Label::new(None);
        status_label.add_css_class("dim-label");
        status_label.set_halign(gtk::Align::Start);

        let extra_box = gtk::Box::new(gtk::Orientation::Vertical, 8);

        let (input_section, _) =
            make_section(opts.input_label, input_view.widget(), opts.input_height);
        let (output_section, output_scroll) =
            make_section(opts.output_label, output_view.widget(), opts.output_height);

        root.append(&input_section);
        root.append(&extra_box);
        root.append(&buttons_box);
        root.append(&output_section);
        root.append(&status_label);

        let inner = Rc::new(Inner {
            input_view,
            output_view,
            output_scroll,
            buttons_box,
            extra_box,
            status_label,
        });
        let owned = Rc::clone(&inner);
        root.connect_destroy(move |_| {
            let _ = &owned;
        });
        Self { inner, root }
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

    #[allow(dead_code)]
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
        self.inner.output_scroll.remove_css_class("dt-error-frame");
        self.inner.output_view.set_text(text);
    }

    pub fn set_error(&self, message: &str) {
        self.inner
            .output_view
            .widget()
            .add_css_class("dt-error-text");
        self.inner.output_scroll.add_css_class("dt-error-frame");
        self.inner
            .output_view
            .set_text(&format!("⚠ Error: {message}"));
        self.inner.status_label.set_text("");
    }

    pub fn set_status(&self, text: &str) {
        self.inner.status_label.set_text(text);
    }

    /// Reports a transform's outcome: the output plus "<verb> at HH:MM:SS",
    /// or the error message.
    pub fn report(&self, verb: &str, result: Result<String, String>) {
        match result {
            Ok(out) => {
                self.set_output(&out);
                self.set_status(&format!("{verb} at {}", timestamp()));
            }
            Err(e) => self.set_error(&e),
        }
    }

    fn copy_output(&self) {
        let buf = self.inner.output_view.buffer();
        if buf.char_count() == 0 {
            return;
        }
        let (start, end) = buf.bounds();
        buf.select_range(&start, &end);
        buf.copy_clipboard(&self.inner.output_view.widget().clipboard());
        let end = buf.end_iter();
        buf.select_range(&end, &end);
        self.set_status(&format!("Copied to clipboard at {}", timestamp()));
    }

    fn clear(&self) {
        self.inner.input_view.set_text("");
        self.set_output("");
        self.inner.status_label.set_text("");
    }

    pub fn into_widget(self) -> gtk::Widget {
        self.add_button_with_class("Copy output", None, Self::copy_output);
        self.add_button_with_class("Clear", None, Self::clear);
        self.root.upcast()
    }
}
