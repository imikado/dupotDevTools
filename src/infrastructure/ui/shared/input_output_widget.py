import datetime

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk

from infrastructure.ui.shared.code_view import CodeView


class InputOutputWidget:
    def __init__(
        self,
        input_label="Input",
        output_label="Output",
        input_height=150,
        output_height=250,
        input_language=None,
        output_language=None,
    ):
        self._box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)

        self._input_view = CodeView(editable=True, language=input_language)
        self._output_view = CodeView(editable=False, language=output_language)

        self._buttons_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        self._buttons_box.set_halign(Gtk.Align.CENTER)

        self._status_label = Gtk.Label(label="")
        self._status_label.add_css_class("dim-label")
        self._status_label.set_halign(Gtk.Align.START)

        self._extra_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)

        input_section, _ = self._make_section(input_label, self._input_view.widget, input_height)
        output_section, self._output_scroll = self._make_section(
            output_label, self._output_view.widget, output_height
        )

        self._box.append(input_section)
        self._box.append(self._extra_box)
        self._box.append(self._buttons_box)
        self._box.append(output_section)
        self._box.append(self._status_label)

        self._utility_buttons_added = False

    def _make_section(self, label_text, view_widget, height):
        section = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=4)
        lbl = Gtk.Label(label=label_text, xalign=0)
        lbl.add_css_class("heading")
        scroll = Gtk.ScrolledWindow()
        scroll.set_size_request(-1, height)
        scroll.set_child(view_widget)
        scroll.add_css_class("card")
        section.append(lbl)
        section.append(scroll)
        return section, scroll

    def _add_utility_buttons(self):
        copy_btn = Gtk.Button(label="Copy output")
        copy_btn.connect("clicked", lambda _: self._copy_output())
        self._buttons_box.append(copy_btn)

        clear_btn = Gtk.Button(label="Clear")
        clear_btn.connect("clicked", lambda _: self.clear())
        self._buttons_box.append(clear_btn)

    def _copy_output(self):
        buf = self._output_view.get_buffer()
        if buf.get_char_count() == 0:
            return
        buf.select_range(buf.get_start_iter(), buf.get_end_iter())
        buf.copy_clipboard(self._output_view.widget.get_clipboard())
        buf.select_range(buf.get_end_iter(), buf.get_end_iter())
        self.set_status(f"Copied to clipboard at {self.timestamp()}")

    def clear(self):
        self._input_view.set_text("")
        self.set_output("")
        self._status_label.set_text("")

    def add_button(self, label, callback, css_class="suggested-action"):
        btn = Gtk.Button(label=label)
        if css_class:
            btn.add_css_class(css_class)
        btn.connect("clicked", lambda _: callback())
        self._buttons_box.append(btn)
        return btn

    def add_extra_widget(self, widget):
        self._extra_box.append(widget)

    def get_input(self) -> str:
        return self._input_view.get_text()

    def set_input(self, text: str):
        self._input_view.set_text(text)

    def set_input_language(self, language):
        self._input_view.set_language(language)

    def set_output_language(self, language):
        self._output_view.set_language(language)

    def set_output(self, text: str):
        self._output_view.widget.remove_css_class("dt-error-text")
        self._output_scroll.remove_css_class("dt-error-frame")
        self._output_view.set_text(text)

    def set_error(self, message: str):
        self._output_view.widget.add_css_class("dt-error-text")
        self._output_scroll.add_css_class("dt-error-frame")
        self._output_view.set_text(f"⚠ Error: {message}")
        self._status_label.set_text("")

    def set_status(self, text: str):
        self._status_label.set_text(text)

    def get_widget(self):
        if not self._utility_buttons_added:
            self._add_utility_buttons()
            self._utility_buttons_added = True
        return self._box

    @staticmethod
    def timestamp() -> str:
        return datetime.datetime.now().strftime("%H:%M:%S")
