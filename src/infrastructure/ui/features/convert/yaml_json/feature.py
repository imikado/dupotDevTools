import json

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(
            input_label="Input",
            output_label="Output",
            input_language="yaml",
            output_language="json",
        )

        def yaml_to_json():
            tool.set_input_language("yaml")
            try:
                import yaml
                data = yaml.safe_load(tool.get_input())
                tool.set_output_language("json")
                tool.set_output(json.dumps(data, indent=4, ensure_ascii=False))
                tool.set_status(f"Converted at {InputOutputWidget.timestamp()}")
            except ImportError:
                tool.set_error("pyyaml is not installed. Run: pip install pyyaml")
            except Exception as e:
                tool.set_error(str(e))

        def json_to_yaml():
            tool.set_input_language("json")
            try:
                import yaml
                data = json.loads(tool.get_input())
                tool.set_output_language("yaml")
                tool.set_output(yaml.dump(data, allow_unicode=True, default_flow_style=False))
                tool.set_status(f"Converted at {InputOutputWidget.timestamp()}")
            except ImportError:
                tool.set_error("pyyaml is not installed. Run: pip install pyyaml")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("YAML → JSON", yaml_to_json)
        tool.add_button("JSON → YAML", json_to_yaml, css_class="")
        return tool.get_widget()
