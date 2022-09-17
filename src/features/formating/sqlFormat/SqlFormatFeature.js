import React, { useState } from "react";
import { Button, Form } from "react-bootstrap";

import DatetimeApi from "../../../apis/DatetimeApi";
import { format } from 'sql-formatter';

const datetimeApi=new DatetimeApi();

export default function SqlFormatFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {

    var outputConverted=format(input,{
      language: 'spark',
      tabWidth: 2,
      keywordCase: 'upper',
      linesBetweenQueries: 2,
    })
    
    setOutput('');
    try{
      setOutput(outputConverted);
    }catch(e){
      console.log('Error'+e.message);
    }
    setStatus('Formated at '+datetimeApi.getTimeToString());
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
          <Form.Text className="text-muted">SQL to format</Form.Text>
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
