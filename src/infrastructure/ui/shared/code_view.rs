//! A code-aware text view backed by GtkSourceView (syntax highlighting, line
//! numbers, current-line highlight) whose style scheme follows the dark mode.

use adw::prelude::*;
use gtk::glib;
use sourceview5::prelude::*;

#[derive(Clone)]
pub struct CodeView {
    buffer: sourceview5::Buffer,
    view: sourceview5::View,
}

fn apply_style_scheme(buffer: &sourceview5::Buffer) {
    let dark = adw::StyleManager::default().is_dark();
    let id = if dark { "Adwaita-dark" } else { "Adwaita" };
    if let Some(scheme) = sourceview5::StyleSchemeManager::default().scheme(id) {
        buffer.set_style_scheme(Some(&scheme));
    }
}

impl CodeView {
    pub fn new(editable: bool, language: Option<&str>) -> Self {
        let buffer = sourceview5::Buffer::new(None);
        let view = sourceview5::View::with_buffer(&buffer);
        view.set_show_line_numbers(false);
        view.set_tab_width(4);
        view.set_insert_spaces_instead_of_tabs(true);
        view.set_monospace(true);
        view.set_wrap_mode(gtk::WrapMode::WordChar);
        view.set_editable(editable);
        view.set_left_margin(8);
        view.set_right_margin(8);
        view.set_top_margin(8);
        view.set_bottom_margin(8);

        let this = Self { buffer, view };
        this.set_language(language);
        apply_style_scheme(&this.buffer);

        let handler = adw::StyleManager::default().connect_dark_notify(glib::clone!(
            #[weak(rename_to = buffer)]
            this.buffer,
            move |_| apply_style_scheme(&buffer)
        ));
        let handler = std::cell::Cell::new(Some(handler));
        this.view.connect_destroy(move |_| {
            if let Some(h) = handler.take() {
                adw::StyleManager::default().disconnect(h);
            }
        });
        this
    }

    pub fn set_language(&self, language: Option<&str>) {
        let lang = language.and_then(|l| sourceview5::LanguageManager::default().language(l));
        self.buffer.set_language(lang.as_ref());
    }

    pub fn text(&self) -> String {
        let (start, end) = self.buffer.bounds();
        self.buffer.text(&start, &end, false).into()
    }

    pub fn set_text(&self, text: &str) {
        self.buffer.set_text(text);
    }

    pub fn widget(&self) -> &sourceview5::View {
        &self.view
    }

    pub fn connect_changed(&self, callback: impl Fn() + 'static) {
        self.buffer.connect_changed(move |_| callback());
    }

    /// Highlights `ranges` (byte offsets into the text), replacing any
    /// previous highlight.
    pub fn highlight(&self, ranges: &[(usize, usize)]) {
        const TAG: &str = "dt-highlight";
        let tag = self.buffer.tag_table().lookup(TAG).unwrap_or_else(|| {
            let tag = gtk::TextTag::builder()
                .name(TAG)
                .background_rgba(&gtk::gdk::RGBA::new(0.96, 0.83, 0.18, 0.45))
                .build();
            self.buffer.tag_table().add(&tag);
            tag
        });
        let (start, end) = self.buffer.bounds();
        self.buffer.remove_tag(&tag, &start, &end);

        let text = self.text();
        let char_offset = |byte: usize| text[..byte.min(text.len())].chars().count() as i32;
        for &(from, to) in ranges {
            if from == to || !text.is_char_boundary(from) || !text.is_char_boundary(to) {
                continue;
            }
            let start = self.buffer.iter_at_offset(char_offset(from));
            let end = self.buffer.iter_at_offset(char_offset(to));
            self.buffer.apply_tag(&tag, &start, &end);
        }
    }
}
