import React, { useEffect, useState } from "react";
import { Button, Card, Col, Form, Row } from "react-bootstrap";
import featuresApi from "../../../apis/featuresApi";
import Settings from "./Settings";

const Convert = require('ansi-to-html');


export default function GetTldr() {

  const section='helpful';
  const feature='tldr';

  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({binaryPath:"aa"});


  //command
  const command= settingsObj.binaryPath; 

  //settings
  const [show, setShow] = useState(false);
  const handleShow = () => setShow(true);
  const handleClose = () => setShow(false);




  const saveSettings=(settingsObj)=>{
    featuresApi.saveJsonSettings(section,feature,settingsObj);
  }

  useEffect(() => {
    setSettingsObj(featuresApi.readJsonSettings(section,feature));
    
  },[]);

  const convert = () => {

    let outputError=featuresApi.launchCommand(command+ " "+input);


    var convert = new Convert({
      newline: true,
     })
      setOutput(convert.toHtml(outputError));

    setStatus("Displayed at " + featuresApi.getTimeToString());
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
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Enter text to convert"
          />
          <Form.Text className="text-muted">Doc to display</Form.Text>
        </Form.Group>

        <Form.Group className="mb-3">
          <Form.Label>Output</Form.Label>
          <Card>
            <Card.Body  dangerouslySetInnerHTML={{__html: output}} />
          </Card>

          <Form.Text className="text-muted">{command+" "+input}</Form.Text>
          <br/>
          <Form.Text className="text-muted">{status}</Form.Text>
        </Form.Group>

        <Button onClick={() => convert()}>Convert</Button>
      </Form>

      <Settings show={show} handleClose={handleClose} handleSave={saveSettings} settingsObj={settingsObj} handleSetSettingsObj={setSettingsObj} />
      
    </div>
  );
}
