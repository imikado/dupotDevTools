import React, { useState } from "react";
import { Button, Form } from "react-bootstrap";

import DatetimeApi from "../../../apis/DatetimeApi";
import TextApi from "../../../apis/TextApi";

const textApi=new TextApi();
const datetimeApi=new DatetimeApi();

export default function JsonFormatFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {

    var outputConverted=textApi.jsonIndent(input);

    setOutput('');
    try{
      setOutput(outputConverted);
    }catch(e){
      console.log('Error'+e.message);
    }
    setStatus('Converted at '+datetimeApi.getTimeToString());
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
          <Form.Control as="textarea" rows={10} value={output} readOnly></Form.Control>
          <Form.Text className="text-muted">{status}</Form.Text>
        </Form.Group>

        <Button onClick={() => convert()}>Convert</Button>
      </Form>
    </div>
  );
}
