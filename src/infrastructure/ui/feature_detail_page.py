import gi

gi.require_version("Gtk", "4.0")
gi.require_version("Adw", "1")
from gi.repository import Gtk, Adw


class FeatureDetailPage(Adw.NavigationPage):
    def __init__(self, feature_entity, feature_instance):
        super().__init__()
        self.set_title(feature_entity.name.replace("_", " ").title())

        toolbar_view = Adw.ToolbarView()
        header_bar = Adw.HeaderBar()
        toolbar_view.add_top_bar(header_bar)

        scroll = Gtk.ScrolledWindow()
        scroll.set_vexpand(True)
        scroll.set_hexpand(True)

        clamp = Adw.Clamp()
        clamp.set_maximum_size(860)
        clamp.set_tightening_threshold(640)
        clamp.set_margin_top(24)
        clamp.set_margin_bottom(24)
        clamp.set_margin_start(12)
        clamp.set_margin_end(12)
        clamp.set_child(feature_instance.get_widget())

        scroll.set_child(clamp)
        toolbar_view.set_content(scroll)
        self.set_child(toolbar_view)
