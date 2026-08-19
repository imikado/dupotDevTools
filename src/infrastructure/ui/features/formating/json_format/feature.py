import json

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(
            input_label="Input JSON",
            output_label="Output",
            input_language="json",
            output_language="json",
        )

        def do_format():
            try:
                result = json.dumps(
                    json.loads(tool.get_input()), indent=4, ensure_ascii=False
                )
                tool.set_output(result)
                tool.set_status(f"Formatted at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Format", do_format)
        return tool.get_widget()
