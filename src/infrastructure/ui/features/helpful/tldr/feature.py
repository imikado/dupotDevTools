import datetime

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk

from domain.contract.feature_contract import FeatureContract
from infrastructure.api.system_api import SystemApi
from infrastructure.ui.shared.code_view import CodeView


class Feature(FeatureContract):
    def get_widget(self):
        _sys = SystemApi()
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)

        # Command entry
        cmd_lbl = Gtk.Label(label="Command", xalign=0)
        cmd_lbl.add_css_class("heading")

        entry_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        entry = Gtk.Entry()
        entry.set_placeholder_text("e.g. git, curl, tar, rsync…")
        entry.set_hexpand(True)
        lookup_btn = Gtk.Button(label="Lookup")
        lookup_btn.add_css_class("suggested-action")
        entry_box.append(entry)
        entry_box.append(lookup_btn)

        # Output
        output_lbl = Gtk.Label(label="Result", xalign=0)
        output_lbl.add_css_class("heading")

        output_scroll = Gtk.ScrolledWindow()
        output_scroll.set_size_request(-1, 350)
        output_scroll.set_vexpand(True)
        output_scroll.add_css_class("card")
        output_view = CodeView(editable=False, language="markdown")
        output_scroll.set_child(output_view.widget)

        btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        btn_box.set_halign(Gtk.Align.CENTER)
        copy_btn = Gtk.Button(label="Copy output")
        btn_box.append(copy_btn)

        status_lbl = Gtk.Label(label="")
        status_lbl.add_css_class("dim-label")
        status_lbl.set_halign(Gtk.Align.START)

        def set_error(message: str):
            status_lbl.remove_css_class("dim-label")
            status_lbl.add_css_class("error")
            status_lbl.set_text("")
            output_view.set_text(f"⚠ Error: {message}")

        def set_status(message: str):
            status_lbl.remove_css_class("error")
            status_lbl.add_css_class("dim-label")
            status_lbl.set_text(message)

        def do_lookup(_widget=None):
            cmd = entry.get_text().strip()
            if not cmd:
                return
            if not _sys.binary_exists("tldr"):
                set_error("'tldr' not found. Install with: pip install tldr   or   npm install -g tldr")
                return
            stdout, stderr = _sys.run_command(f"tldr {cmd}", timeout=15)
            if stdout:
                output_view.set_text(stdout)
                set_status(f"Retrieved at {datetime.datetime.now().strftime('%H:%M:%S')}")
            else:
                set_error(stderr or "no result")

        def do_copy(_btn):
            buf = output_view.get_buffer()
            if buf.get_char_count() == 0:
                return
            buf.select_range(buf.get_start_iter(), buf.get_end_iter())
            buf.copy_clipboard(output_view.widget.get_clipboard())
            buf.select_range(buf.get_end_iter(), buf.get_end_iter())
            set_status(f"Copied to clipboard at {datetime.datetime.now().strftime('%H:%M:%S')}")

        entry.connect("activate", do_lookup)
        lookup_btn.connect("clicked", do_lookup)
        copy_btn.connect("clicked", do_copy)

        box.append(cmd_lbl)
        box.append(entry_box)
        box.append(output_lbl)
        box.append(output_scroll)
        box.append(btn_box)
        box.append(status_lbl)
        return box
