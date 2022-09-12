import React, { useState } from 'react';
import { Button, Form } from 'react-bootstrap';

import TextApi from '../../../apis/TextApi';
import DatetimeApi from '../../../apis/DatetimeApi';

const textApi=new TextApi();
const datetimeApi=new DatetimeApi();


export default function Hash() {
  const [input, setInput] = useState("");
  const [outputMd5, setOutputMd5] = useState("");
  const [outputSha1, setOutputSha1] = useState("");
  const [outputSha256, setOutputSha256] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {
   
    setOutputMd5(textApi.hash(input,'md5'))
    setOutputSha1(textApi.hash(input,'sha1'))
    setOutputSha256(textApi.hash(input,'sha256'))

    setStatus('Converted at '+datetimeApi.getTimeToString());
  };

  const resetOutput= () => {
    setOutputMd5('')
    setOutputSha1('')
    console.log('reset')
  }

  return (
    <div>
      <Form>
        <Form.Group className="mb-3">
          <Form.Label>Input</Form.Label>
          <Form.Control
             value={input}
            onChange={(e) => {
              setInput(e.target.value)
              resetOutput()
            }}
            placeholder="Enter text to hash"
          />
          <Form.Text className="text-muted">String you want to hash</Form.Text>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Md5</Form.Label>
          <Form.Control  value={outputMd5} readOnly></Form.Control>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Sha1</Form.Label>
          <Form.Control  value={outputSha1} readOnly></Form.Control>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Sha256</Form.Label>
          <Form.Control  value={outputSha256} readOnly></Form.Control>
        </Form.Group>

        <Button onClick={() => convert()}>Convert</Button>
      </Form>
    </div>
  );
}
