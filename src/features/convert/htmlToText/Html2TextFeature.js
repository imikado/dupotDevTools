import React, { useEffect, useState } from "react";
import { Button, Col, Form, Row } from "react-bootstrap";

import SystemApi from "../../../apis/SystemApi";
import DatetimeApi from "../../../apis/DatetimeApi";

import Settings from "./Settings";
import card from "./card.json";

const datetimeApi=new DatetimeApi();

const systemApi=new SystemApi();
systemApi.loadCard(card);

export default function Html2TextFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({binaryPath:"aa"});


  //command
  const inputFile = systemApi.getTempFilePath("input");
  const outputFile = systemApi.getTempFilePath("output");
  const command= settingsObj.binaryPath+ " -o " + outputFile+" " + inputFile 

  //settings
  const [show, setShow] = useState(false);
  const handleShow = () => setShow(true);
  const handleClose = () => setShow(false);




  const saveSettings=(settingsObj)=>{
    systemApi.saveJsonSettings(settingsObj);
  }

  useEffect(() => {
    setSettingsObj(systemApi.readJsonSettings());
    
  },[]);

  const convert = () => {
    systemApi.writeTempFile(inputFile, input);

    let outputError=systemApi.launchCommand(command);

    var outputConverted = systemApi.readTempFile(outputFile);

    if(outputError){
      setOutput(outputError);
    
    }else{
      setOutput(outputConverted);
    
    }
    setStatus("Converted at " + datetimeApi.getTimeToString());
  };

  return (
    <div>
      <Row>
        <Col></Col>
        <Col align="end"><Button variant="default" onClick={handleShow}>Settings</Button></Col>
      </Row>
      
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
          <Form.Control
            as="textarea"
            rows={9}
            value={output}
            readOnly
          ></Form.Control>
          <Form.Text className="text-muted">{command}</Form.Text>
          <br/>
          <Form.Text className="text-muted">{status}</Form.Text>
        </Form.Group>

        <Button onClick={() => convert()}>Convert</Button>
      </Form>

      <Settings show={show} handleClose={handleClose} handleSave={saveSettings} settingsObj={settingsObj} handleSetSettingsObj={setSettingsObj} />
      
    </div>
  );
}
