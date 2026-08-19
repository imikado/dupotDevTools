from abc import ABC, abstractmethod


class FeatureContract(ABC):
    @abstractmethod
    def get_widget(self):
        """Returns a Gtk.Widget for this feature's UI."""
        pass
