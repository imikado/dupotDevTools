import React, { useState } from "react";
import { Button, Form } from "react-bootstrap";
import featuresApi from "../../../apis/featuresApi";

export default function YamlToJson() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convertToYaml = () => {
    setOutput(featuresApi.convertJsonToYaml(input));

    setStatus('Converted at '+featuresApi.getTimeToString());
  };
  const convertToJson = () => {
    setOutput(featuresApi.convertYamlToJson(input));

    setStatus('Converted at '+featuresApi.getTimeToString());
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
