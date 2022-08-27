import React, { useState } from "react";
import { Button, Form } from "react-bootstrap";
import featuresApi from "../../../apis/featuresApi";

export default function Html2Text() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {
    const inputFile = featuresApi.getTempFilePath("html2text.input");
    featuresApi.writeTempFile(inputFile, input);

    const outputFile = featuresApi.getTempFilePath("html2text.output");
    featuresApi.launchCommand("html2text " + inputFile + ' > '+outputFile  );

    var outputConverted=featuresApi.readTempFile(outputFile);

    setOutput(outputConverted);
    setStatus('Converted at '+featuresApi.getTimeToString());
  };

  return (
    <div>
      <Form>
        <Form.Group className="mb-3">
          <Form.Label>Input</Form.Label>
          <Form.Control
            as="textarea"
            rows={3}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Enter text to convert"
          />
          <Form.Text className="text-muted">HTML to convert to text</Form.Text>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Output</Form.Label>
          <Form.Control as="textarea" rows={3} value={output} readOnly></Form.Control>
          <Form.Text className="text-muted">{status}</Form.Text>
        </Form.Group>

        <Button onClick={() => convert()}>Convert</Button>
      </Form>
    </div>
  );
}
