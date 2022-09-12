import React, { useState } from 'react';
import { Button, Form } from 'react-bootstrap';
import DatetimeApi from '../../../apis/DatetimeApi';

const datetimeApi=new DatetimeApi();

export default function UrlEncodingFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  
  const [status, setStatus] = useState();

  const encode = () => {
   
    setOutput(encodeURI(input))

    setStatus('Encoding at '+datetimeApi.getTimeToString());
  };
  const decode = () => {
   
    setOutput(decodeURI(input))

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
          <Form.Text className="text-muted">String you want to encode/decode from/to URL</Form.Text>
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
