from abc import ABC, abstractmethod


class FeatureContract(ABC):
    #: Set to True on a subclass to opt out of the detail page's width clamp
    #: (e.g. for canvas/table-like UIs that need the full window width).
    FULL_WIDTH = False

    @abstractmethod
    def get_widget(self):
        """Returns a Gtk.Widget for this feature's UI."""
        pass
