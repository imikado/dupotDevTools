import os

import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Adw, Gdk, GdkPixbuf, Gio, GLib, Gtk

from infrastructure.service.feature_loader_service import FeatureLoaderService
from infrastructure.ui.feature_detail_page import FeatureDetailPage

FEATURES_DIR = os.path.join(os.path.dirname(__file__), "features")

_APP_CSS = """
.dt-error-frame {
    border: 2px solid @error_color;
    border-radius: 6px;
}

textview.dt-error-text text {
    color: @error_color;
}
"""


def _install_app_css():
    provider = Gtk.CssProvider()
    provider.load_from_string(_APP_CSS)
    Gtk.StyleContext.add_provider_for_display(
        Gdk.Display.get_default(), provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
    )

def _recolor_pixbuf(pixbuf, rgb_hex):
    """Flatten an icon's colors to a single tint, keeping its alpha mask.

    Feature icons ship as dark silhouettes on transparent backgrounds, so
    they vanish against a dark theme. Recoloring them like symbolic icons
    (single flat color driven by the current theme) keeps them legible
    without needing a background chip behind each one.
    """
    if not pixbuf.get_has_alpha():
        pixbuf = pixbuf.add_alpha(False, 0, 0, 0)

    r = int(rgb_hex[1:3], 16)
    g = int(rgb_hex[3:5], 16)
    b = int(rgb_hex[5:7], 16)

    width = pixbuf.get_width()
    height = pixbuf.get_height()
    rowstride = pixbuf.get_rowstride()
    n_channels = pixbuf.get_n_channels()

    data = bytearray(pixbuf.get_pixels())
    for y in range(height):
        row = y * rowstride
        for x in range(width):
            idx = row + x * n_channels
            data[idx] = r
            data[idx + 1] = g
            data[idx + 2] = b

    return GdkPixbuf.Pixbuf.new_from_bytes(
        GLib.Bytes.new(bytes(data)),
        GdkPixbuf.Colorspace.RGB,
        True,
        8,
        width,
        height,
        rowstride,
    )


_SECTION_ICONS = {
    "convert": "media-playlist-shuffle-symbolic",
    "encoding": "security-high-symbolic",
    "formating": "format-justify-fill-symbolic",
    "graphics": "image-x-generic-symbolic",
    "helpful": "help-contents-symbolic",
    "parsing": "text-x-generic-symbolic",
    "string": "insert-text-symbolic",
    "devices": "computer-symbolic",
}


class MainWindow(Adw.ApplicationWindow):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.set_title("DevTools")
        self.set_default_size(1100, 700)

        self._loader = FeatureLoaderService(FEATURES_DIR)
        self._sections = self._loader.get_sections()

        self._nav_view = Adw.NavigationView()
        self.set_content(self._nav_view)

        self._content_bin = Adw.Bin()
        self._content_bin.set_hexpand(True)
        self._content_bin.set_vexpand(True)

        self._nav_view.push(self._build_home_page())

        if self._sections:
            self._show_section(self._sections[0])

    def _build_home_page(self):
        toolbar_view = Adw.ToolbarView()

        header_bar = Adw.HeaderBar()
        header_bar.set_title_widget(Gtk.Label(label="DevTools"))

        menu = Gio.Menu()
        menu.append("About", "win.about")
        menu_btn = Gtk.MenuButton()
        menu_btn.set_icon_name("open-menu-symbolic")
        menu_btn.set_menu_model(menu)
        header_bar.pack_end(menu_btn)

        about_action = Gio.SimpleAction.new("about", None)
        about_action.connect("activate", self._on_about)
        self.add_action(about_action)

        toolbar_view.add_top_bar(header_bar)

        main_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=0)
        main_box.append(self._build_sidebar())
        main_box.append(Gtk.Separator(orientation=Gtk.Orientation.VERTICAL))
        main_box.append(self._content_bin)

        toolbar_view.set_content(main_box)
        return Adw.NavigationPage.new(toolbar_view, "DevTools")

    def _build_sidebar(self):
        box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=0)
        box.set_size_request(220, -1)

        lbl = Gtk.Label(label="Sections", xalign=0)
        lbl.add_css_class("heading")
        lbl.set_margin_top(16)
        lbl.set_margin_bottom(8)
        lbl.set_margin_start(16)
        box.append(lbl)

        self._sidebar_list = Gtk.ListBox()
        self._sidebar_list.set_selection_mode(Gtk.SelectionMode.SINGLE)
        self._sidebar_list.add_css_class("navigation-sidebar")
        self._sidebar_list.connect("row-activated", self._on_section_activated)

        for section in self._sections:
            row = Gtk.ListBoxRow()
            row._section = section

            row_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
            row_box.set_margin_start(12)
            row_box.set_margin_end(12)
            row_box.set_margin_top(8)
            row_box.set_margin_bottom(8)

            icon = Gtk.Image.new_from_icon_name(
                _SECTION_ICONS.get(section, "applications-utilities-symbolic")
            )
            label = Gtk.Label(label=section.replace("_", " ").title())
            label.set_halign(Gtk.Align.START)
            label.set_hexpand(True)

            row_box.append(icon)
            row_box.append(label)
            row.set_child(row_box)
            self._sidebar_list.append(row)

        scroll = Gtk.ScrolledWindow()
        scroll.set_vexpand(True)
        scroll.set_child(self._sidebar_list)
        box.append(scroll)

        first = self._sidebar_list.get_row_at_index(0)
        if first:
            self._sidebar_list.select_row(first)

        return box

    def _on_section_activated(self, _list, row):
        self._show_section(row._section)

    def _show_section(self, section: str):
        features = self._loader.get_features_in_section(section)

        flow = Gtk.FlowBox()
        flow.set_valign(Gtk.Align.START)
        flow.set_max_children_per_line(4)
        flow.set_min_children_per_line(2)
        flow.set_selection_mode(Gtk.SelectionMode.NONE)
        flow.set_row_spacing(12)
        flow.set_column_spacing(12)
        flow.set_margin_start(24)
        flow.set_margin_end(24)
        flow.set_margin_top(24)
        flow.set_margin_bottom(24)

        for feature in features:
            flow.append(self._build_feature_card(feature))

        scroll = Gtk.ScrolledWindow()
        scroll.set_vexpand(True)
        scroll.set_hexpand(True)
        scroll.set_child(flow)
        self._content_bin.set_child(scroll)

    def _make_feature_icon(self, icon_path, size=56):
        if icon_path and os.path.exists(icon_path):
            try:
                pixbuf = GdkPixbuf.Pixbuf.new_from_file_at_scale(
                    icon_path, size, size, True
                )
                dark = Adw.StyleManager.get_default().get_dark()
                pixbuf = _recolor_pixbuf(pixbuf, "#ffffff" if dark else "#2e3436")
                icon = Gtk.Image.new_from_pixbuf(pixbuf)
                icon.set_size_request(size, size)
                return icon
            except GLib.Error:
                pass
        icon = Gtk.Image.new_from_icon_name("applications-utilities-symbolic")
        icon.set_pixel_size(size)
        icon.set_size_request(size, size)
        return icon

    def _build_feature_card(self, feature):
        button = Gtk.Button()
        button.add_css_class("card")
        button.set_size_request(190, 180)
        button.connect("clicked", lambda _b: self._on_feature_clicked(feature))

        content = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        content.set_halign(Gtk.Align.CENTER)
        content.set_valign(Gtk.Align.CENTER)
        content.set_margin_top(16)
        content.set_margin_bottom(16)
        content.set_margin_start(12)
        content.set_margin_end(12)

        icon_size = 56
        icon = self._make_feature_icon(feature.icon_path, size=icon_size)
        icon_box = Gtk.Box()
        icon_box.set_size_request(icon_size, icon_size)
        icon_box.set_halign(Gtk.Align.CENTER)
        icon_box.set_valign(Gtk.Align.CENTER)
        icon_box.append(icon)

        name_lbl = Gtk.Label(label=feature.name.replace("_", " ").title())
        name_lbl.add_css_class("title-4")
        name_lbl.set_wrap(True)
        name_lbl.set_justify(Gtk.Justification.CENTER)

        desc_lbl = Gtk.Label(label=feature.description)
        desc_lbl.add_css_class("dim-label")
        desc_lbl.add_css_class("caption")
        desc_lbl.set_wrap(True)
        desc_lbl.set_justify(Gtk.Justification.CENTER)
        desc_lbl.set_max_width_chars(22)

        content.append(icon_box)
        content.append(name_lbl)
        content.append(desc_lbl)
        button.set_child(content)
        return button

    def _on_feature_clicked(self, feature):
        try:
            instance = self._loader.load_feature_instance(feature)
            self._nav_view.push(FeatureDetailPage(feature, instance))
        except Exception as e:
            print(f"Error loading feature {feature.name}: {e}")
            import traceback
            traceback.print_exc()

    def _on_about(self, _action, _param):
        about = Adw.AboutDialog.new()
        about.set_application_name("DevTools")
        about.set_version("1.0.0")
        about.set_developer_name("dupot")
        about.set_license_type(Gtk.License.LGPL_2_1)
        about.present(self)


class AppWindow(Adw.Application):
    def __init__(self):
        super().__init__(
            application_id="org.dupot.devtools",
            flags=Gio.ApplicationFlags.DEFAULT_FLAGS,
        )

    def do_activate(self):
        _install_app_css()
        win = MainWindow(application=self)
        win.present()
