import React, { useEffect, useState } from "react";
import { Button, Col, Form, Row } from "react-bootstrap";

import FileSystemApi from "../../../apis/FileSystemApi";
import DatetimeApi from "../../../apis/DatetimeApi";

import Settings from "./Settings";
import card from "./card.json";

const datetimeApi=new DatetimeApi();

const filesystemApi=new FileSystemApi();
filesystemApi.loadCard(card);

export default function Html2Text() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({binaryPath:"aa"});


  //command
  const inputFile = filesystemApi.getTempFilePath("input");
  const outputFile = filesystemApi.getTempFilePath("output");
  const command= settingsObj.binaryPath+ " -o " + outputFile+" " + inputFile 

  //settings
  const [show, setShow] = useState(false);
  const handleShow = () => setShow(true);
  const handleClose = () => setShow(false);




  const saveSettings=(settingsObj)=>{
    filesystemApi.saveJsonSettings(settingsObj);
  }

  useEffect(() => {
    setSettingsObj(filesystemApi.readJsonSettings());
    
  },[]);

  const convert = () => {
    filesystemApi.writeTempFile(inputFile, input);

    let outputError=filesystemApi.launchCommand(command);

    var outputConverted = filesystemApi.readTempFile(outputFile);

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
