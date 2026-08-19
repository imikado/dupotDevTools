import xml.dom.minidom

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(
            input_label="Input XML",
            output_label="Output",
            input_language="xml",
            output_language="xml",
        )

        def do_format():
            try:
                dom = xml.dom.minidom.parseString(tool.get_input().strip())
                result = dom.toprettyxml(indent="    ")
                # Remove extra blank lines added by toprettyxml
                lines = [l for l in result.splitlines() if l.strip()]
                tool.set_output("\n".join(lines))
                tool.set_status(f"Formatted at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Format", do_format)
        return tool.get_widget()
