import React, { useState } from "react";
import { Button, Form } from "react-bootstrap";
import DatetimeApi from "../../../apis/DatetimeApi";
import TextApi from "../../../apis/TextApi";

const datetimeApi=new DatetimeApi();
const textApi=new TextApi();

export default function YamlToJsonFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convertToYaml = () => {
    setOutput(textApi.convertJsonToYaml(input));

    setStatus('Converted at '+datetimeApi.getTimeToString());
  };
  const convertToJson = () => {
    setOutput(textApi.convertYamlToJson(input));

    setStatus('Converted at '+datetimeApi.getTimeToString());
  };

  return (
    <div>
      <Form>
        <Form.Group className="mb-3">
          <Form.Label>Input</Form.Label>
          <Form.Control
            as="textarea"
            rows={5}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Enter text to convert"
          />
          <Form.Text className="text-muted">YAML/JSON to convert to JSON/YAML</Form.Text>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Output</Form.Label>
          <Form.Control as="textarea" rows={8} value={output} readOnly></Form.Control>
          <Form.Text className="text-muted">{status}</Form.Text>
        </Form.Group>

        <Button onClick={() => convertToYaml()}>Convert to YAML</Button>
        &nbsp;
        <Button onClick={() => convertToJson()}>Convert to JSON</Button>

      </Form>
    </div>
  );
}
