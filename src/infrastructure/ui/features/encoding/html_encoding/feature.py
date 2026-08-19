import html

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(input_language="html", output_language="html")

        def do_encode():
            try:
                result = html.escape(tool.get_input())
                tool.set_output(result)
                tool.set_status(f"Encoded at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        def do_decode():
            try:
                result = html.unescape(tool.get_input())
                tool.set_output(result)
                tool.set_status(f"Decoded at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Encode", do_encode)
        tool.add_button("Decode", do_decode, css_class="")
        return tool.get_widget()
