import datetime
import hashlib

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk

from domain.contract.feature_contract import FeatureContract

_ALGORITHMS = ["md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3_256", "blake2b"]


class Feature(FeatureContract):
    def get_widget(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)

        # Input
        input_lbl = Gtk.Label(label="Input", xalign=0)
        input_lbl.add_css_class("heading")

        input_scroll = Gtk.ScrolledWindow()
        input_scroll.set_size_request(-1, 150)
        input_scroll.add_css_class("card")
        input_tv = Gtk.TextView()
        input_tv.set_monospace(True)
        input_tv.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        input_tv.set_left_margin(8)
        input_tv.set_right_margin(8)
        input_tv.set_top_margin(8)
        input_tv.set_bottom_margin(8)
        input_scroll.set_child(input_tv)

        # Algorithm selector
        algo_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        algo_box.set_halign(Gtk.Align.CENTER)
        algo_lbl = Gtk.Label(label="Algorithm:")
        string_list = Gtk.StringList.new(_ALGORITHMS)
        dropdown = Gtk.DropDown(model=string_list)
        dropdown.set_selected(_ALGORITHMS.index("sha256"))
        algo_box.append(algo_lbl)
        algo_box.append(dropdown)

        # Buttons
        btn_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        btn_box.set_halign(Gtk.Align.CENTER)
        hash_btn = Gtk.Button(label="Hash")
        hash_btn.add_css_class("suggested-action")
        copy_btn = Gtk.Button(label="Copy hash")
        clear_btn = Gtk.Button(label="Clear")
        btn_box.append(hash_btn)
        btn_box.append(copy_btn)
        btn_box.append(clear_btn)

        # Output
        output_lbl = Gtk.Label(label="Hash", xalign=0)
        output_lbl.add_css_class("heading")

        output_scroll = Gtk.ScrolledWindow()
        output_scroll.set_size_request(-1, 80)
        output_scroll.add_css_class("card")
        output_tv = Gtk.TextView()
        output_tv.set_monospace(True)
        output_tv.set_editable(False)
        output_tv.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        output_tv.set_left_margin(8)
        output_tv.set_right_margin(8)
        output_tv.set_top_margin(8)
        output_tv.set_bottom_margin(8)
        output_scroll.set_child(output_tv)

        status_lbl = Gtk.Label(label="")
        status_lbl.add_css_class("dim-label")
        status_lbl.set_halign(Gtk.Align.START)

        def set_error(message: str):
            status_lbl.remove_css_class("dim-label")
            status_lbl.add_css_class("error")
            status_lbl.set_text("")
            output_tv.get_buffer().set_text(f"⚠ Error: {message}")

        def set_status(message: str):
            status_lbl.remove_css_class("error")
            status_lbl.add_css_class("dim-label")
            status_lbl.set_text(message)

        def do_hash(_btn):
            buf = input_tv.get_buffer()
            text = buf.get_text(buf.get_start_iter(), buf.get_end_iter(), False)
            algo = _ALGORITHMS[dropdown.get_selected()]
            try:
                h = hashlib.new(algo)
                h.update(text.encode("utf-8"))
                output_tv.get_buffer().set_text(h.hexdigest())
                set_status(f"{algo.upper()} at {datetime.datetime.now().strftime('%H:%M:%S')}")
            except Exception as e:
                set_error(str(e))

        def do_copy(_btn):
            buf = output_tv.get_buffer()
            if buf.get_char_count() == 0:
                return
            buf.select_range(buf.get_start_iter(), buf.get_end_iter())
            buf.copy_clipboard(output_tv.get_clipboard())
            buf.select_range(buf.get_end_iter(), buf.get_end_iter())
            set_status(f"Copied to clipboard at {datetime.datetime.now().strftime('%H:%M:%S')}")

        def do_clear(_btn):
            input_tv.get_buffer().set_text("")
            output_tv.get_buffer().set_text("")
            set_status("")

        hash_btn.connect("clicked", do_hash)
        copy_btn.connect("clicked", do_copy)
        clear_btn.connect("clicked", do_clear)

        box.append(input_lbl)
        box.append(input_scroll)
        box.append(algo_box)
        box.append(btn_box)
        box.append(output_lbl)
        box.append(output_scroll)
        box.append(status_lbl)
        return box
