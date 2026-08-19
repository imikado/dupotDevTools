from urllib.parse import quote, unquote

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget()

        def do_encode():
            try:
                result = quote(tool.get_input(), safe="")
                tool.set_output(result)
                tool.set_status(f"Encoded at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        def do_decode():
            try:
                result = unquote(tool.get_input())
                tool.set_output(result)
                tool.set_status(f"Decoded at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Encode", do_encode)
        tool.add_button("Decode", do_decode, css_class="")
        return tool.get_widget()
