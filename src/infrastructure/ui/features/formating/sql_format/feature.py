from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(
            input_label="Input SQL",
            output_label="Output",
            input_language="sql",
            output_language="sql",
        )

        def do_format():
            try:
                import sqlparse
                result = sqlparse.format(
                    tool.get_input(),
                    reindent=True,
                    keyword_case="upper",
                    indent_width=4,
                )
                tool.set_output(result)
                tool.set_status(f"Formatted at {InputOutputWidget.timestamp()}")
            except ImportError:
                tool.set_error("sqlparse is not installed. Run: pip install sqlparse")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Format", do_format)
        return tool.get_widget()
