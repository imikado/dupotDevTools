"""A code-aware text view.

Wraps GtkSourceView (syntax highlighting, line numbers, current-line
highlight) when it is available on the system, and degrades to a plain
monospace Gtk.TextView otherwise. Callers use the same small API
(get_text/set_text/set_language/widget) regardless of which backend is
active, so features never need to know which one they got.
"""

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk

try:
    gi.require_version("GtkSource", "5")
    from gi.repository import GtkSource

    HAS_GTKSOURCE = True
except (ImportError, ValueError):
    HAS_GTKSOURCE = False

try:
    gi.require_version("Adw", "1")
    from gi.repository import Adw
except (ImportError, ValueError):
    Adw = None


def _scheme_id(dark: bool) -> str:
    return "Adwaita-dark" if dark else "Adwaita"


class CodeView:
    def __init__(self, editable: bool = True, language: str | None = None):
        self._language = language
        self._style_handler = None

        if HAS_GTKSOURCE:
            self._buffer = GtkSource.Buffer()
            self._view = GtkSource.View(buffer=self._buffer)
            self._view.set_show_line_numbers(True)
            self._view.set_highlight_current_line(editable)
            self._view.set_tab_width(4)
            self._view.set_insert_spaces_instead_of_tabs(True)
            self._apply_language(language)
            self._apply_style_scheme()
            if Adw is not None:
                style_manager = Adw.StyleManager.get_default()
                self._style_handler = style_manager.connect(
                    "notify::dark", lambda *_: self._apply_style_scheme()
                )
        else:
            self._buffer = Gtk.TextBuffer()
            self._view = Gtk.TextView(buffer=self._buffer)

        self._view.set_monospace(True)
        self._view.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        self._view.set_editable(editable)
        self._view.set_left_margin(8)
        self._view.set_right_margin(8)
        self._view.set_top_margin(8)
        self._view.set_bottom_margin(8)

    def _apply_language(self, language: str | None):
        if not HAS_GTKSOURCE:
            return
        self._buffer.set_language(None)
        if not language:
            return
        lang = GtkSource.LanguageManager.get_default().get_language(language)
        if lang is not None:
            self._buffer.set_language(lang)

    def _apply_style_scheme(self):
        if not HAS_GTKSOURCE:
            return
        dark = Adw.StyleManager.get_default().get_dark() if Adw is not None else False
        scheme = GtkSource.StyleSchemeManager.get_default().get_scheme(_scheme_id(dark))
        if scheme is not None:
            self._buffer.set_style_scheme(scheme)

    def set_language(self, language: str | None):
        self._language = language
        self._apply_language(language)

    def get_text(self) -> str:
        return self._buffer.get_text(
            self._buffer.get_start_iter(), self._buffer.get_end_iter(), False
        )

    def set_text(self, text: str):
        self._buffer.set_text(text)

    def get_buffer(self):
        return self._buffer

    @property
    def widget(self):
        return self._view
