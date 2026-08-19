from html.parser import HTMLParser

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


class _StripHTMLParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self._parts = []

    def handle_data(self, data):
        self._parts.append(data)

    def get_text(self):
        return "".join(self._parts)


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(
            input_label="Input HTML",
            output_label="Plain text",
            input_language="html",
        )

        def do_convert():
            try:
                try:
                    # Try html2text package first (richer output)
                    import html2text as h2t
                    converter = h2t.HTML2Text()
                    converter.ignore_links = False
                    result = converter.handle(tool.get_input())
                except ImportError:
                    # Fallback: stdlib parser
                    parser = _StripHTMLParser()
                    parser.feed(tool.get_input())
                    result = parser.get_text()

                tool.set_output(result)
                tool.set_status(f"Converted at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Convert", do_convert)
        return tool.get_widget()
