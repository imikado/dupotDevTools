import datetime
import shutil

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gio, GLib, Gtk

from domain.contract.feature_contract import FeatureContract
from infrastructure.api.system_api import SystemApi
from infrastructure.ui.shared.code_view import CodeView


class Feature(FeatureContract):
    def get_widget(self):
        _sys = SystemApi()
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)

        # Input
        input_lbl = Gtk.Label(label="DOT Language", xalign=0)
        input_lbl.add_css_class("heading")

        input_scroll = Gtk.ScrolledWindow()
        input_scroll.set_size_request(-1, 150)
        input_scroll.add_css_class("card")
        input_view = CodeView(editable=True, language="dot")
        input_view.set_text('digraph {\n    A -> {B C}\n    B -> D\n    C -> D\n}')
        input_scroll.set_child(input_view.widget)

        # Buttons
        btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        btn_box.set_halign(Gtk.Align.CENTER)
        generate_btn = Gtk.Button(label="Generate")
        generate_btn.add_css_class("suggested-action")
        download_btn = Gtk.Button(label="Download SVG")
        download_btn.set_sensitive(False)
        clear_btn = Gtk.Button(label="Clear")
        btn_box.append(generate_btn)
        btn_box.append(download_btn)
        btn_box.append(clear_btn)

        # Image output
        picture = Gtk.Picture()
        picture.set_can_shrink(True)
        picture.set_content_fit(Gtk.ContentFit.CONTAIN)
        picture.set_size_request(-1, 300)
        picture.set_vexpand(True)

        status_lbl = Gtk.Label(label="")
        status_lbl.add_css_class("dim-label")
        status_lbl.set_halign(Gtk.Align.START)

        svg_path = [None]

        def set_error(message: str):
            status_lbl.remove_css_class("dim-label")
            status_lbl.add_css_class("error")
            status_lbl.set_text(f"⚠ Error: {message}")

        def set_status(message: str):
            status_lbl.remove_css_class("error")
            status_lbl.add_css_class("dim-label")
            status_lbl.set_text(message)

        def generate(_btn):
            dot_src = input_view.get_text()

            if not _sys.binary_exists("dot"):
                set_error("'dot' binary not found. Install graphviz: sudo apt install graphviz")
                return

            in_file = _sys.write_temp_file(dot_src, suffix=".dot")
            out_png = in_file.replace(".dot", ".png")
            out_svg = in_file.replace(".dot", ".svg")

            _, stderr_png = _sys.run_command(f"dot -Tpng '{in_file}' -o '{out_png}'")
            _, stderr_svg = _sys.run_command(f"dot -Tsvg '{in_file}' -o '{out_svg}'")
            _sys.delete_file(in_file)

            if stderr_png:
                set_error(stderr_png[:300])
                download_btn.set_sensitive(False)
            else:
                picture.set_file(Gio.File.new_for_path(out_png))
                svg_path[0] = out_svg
                download_btn.set_sensitive(not bool(stderr_svg))
                set_status(f"Generated at {datetime.datetime.now().strftime('%H:%M:%S')}")

        def download_svg(_btn):
            if not svg_path[0]:
                return
            dialog = Gtk.FileDialog()
            dialog.set_initial_name("graph.svg")
            svg_filter = Gtk.FileFilter()
            svg_filter.set_name("SVG files")
            svg_filter.add_pattern("*.svg")
            filters = Gio.ListStore.new(Gtk.FileFilter)
            filters.append(svg_filter)
            dialog.set_filters(filters)

            def on_save(d, result):
                try:
                    dest = d.save_finish(result)
                    shutil.copy(svg_path[0], dest.get_path())
                    set_status(f"Saved to {dest.get_path()}")
                except GLib.Error:
                    pass

            dialog.save(download_btn.get_root(), None, on_save)

        def clear(_btn):
            input_view.set_text("")
            picture.set_file(None)
            svg_path[0] = None
            download_btn.set_sensitive(False)
            set_status("")

        generate_btn.connect("clicked", generate)
        download_btn.connect("clicked", download_svg)
        clear_btn.connect("clicked", clear)

        box.append(input_lbl)
        box.append(input_scroll)
        box.append(btn_box)
        box.append(picture)
        box.append(status_lbl)
        return box
