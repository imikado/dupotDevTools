import base64
import json

from domain.contract.feature_contract import FeatureContract
from infrastructure.ui.shared.input_output_widget import InputOutputWidget


def _decode_part(part: str) -> dict:
    padding = 4 - len(part) % 4
    if padding != 4:
        part += "=" * padding
    return json.loads(base64.urlsafe_b64decode(part))


class Feature(FeatureContract):
    def get_widget(self):
        tool = InputOutputWidget(
            input_label="JWT Token",
            output_label="Decoded",
            output_language="json",
        )

        def do_decode():
            token = tool.get_input().strip()
            try:
                parts = token.split(".")
                if len(parts) < 2:
                    tool.set_error("not a valid JWT (expected header.payload.signature)")
                    return
                header = _decode_part(parts[0])
                payload = _decode_part(parts[1])
                result = json.dumps(
                    {"header": header, "payload": payload}, indent=4, ensure_ascii=False
                )
                tool.set_output(result)
                tool.set_status(f"Decoded at {InputOutputWidget.timestamp()}")
            except Exception as e:
                tool.set_error(str(e))

        tool.add_button("Decode", do_decode)
        return tool.get_widget()
