import React, { useState } from 'react';
import { Button, Form } from 'react-bootstrap';

import DatetimeApi from "../../../apis/DatetimeApi";
import TextApi from "../../../apis/TextApi";

const datetimeApi=new DatetimeApi();
const textApi=new TextApi();


export default function HtmlEncoding() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  
  const [status, setStatus] = useState();

  const encode = () => {
   
    setOutput(textApi.htmlEncode(input))

    setStatus('Encoding at '+datetimeApi.getTimeToString());
  };
  const decode = () => {
   
    setOutput(textApi.htmlDecode(input))

    setStatus('Decoding at '+datetimeApi.getTimeToString());
  };

  const resetOutput= () => {
    setOutput('')
  }

  return (
    <div>
      <Form>
        <Form.Group className="mb-3">
          <Form.Label>Input</Form.Label>
          <Form.Control
            as="textarea"
            rows={3}
             value={input}
            onChange={(e) => {
              setInput(e.target.value)
              resetOutput()
            }}
            placeholder="Enter text to hash"
          />
          <Form.Text className="text-muted">String you want to encode/decode from/to HTML</Form.Text>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Output</Form.Label>
          <Form.Control as="textarea"
            rows={9} value={output} readOnly></Form.Control>
            <Form.Text className="text-muted">{status}</Form.Text>

        </Form.Group>

      

        <Button onClick={() => encode()}>Encode</Button>
        &nbsp;

        <Button onClick={() => decode()}>Decode</Button>
      </Form>
    </div>
  );
}
