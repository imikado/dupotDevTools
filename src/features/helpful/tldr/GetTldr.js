import React, { useEffect, useState } from "react";
import { Button, Card, Col, Form, Row } from "react-bootstrap";

import Settings from "./Settings";
import card from "./card.json";

import SystemApi from "../../../apis/SystemApi";
import DatetimeApi from "../../../apis/DatetimeApi";

const sytemApi=new SystemApi();
sytemApi.loadCard(card);

const datetimeApi=new DatetimeApi();

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
    sytemApi.saveJsonSettings(settingsObj);
  }

  useEffect(() => {
    setSettingsObj(sytemApi.readJsonSettings());
    
  },[]);

  const convert = () => {

    let outputError=sytemApi.launchCommand(command+ " "+input);


    var convert = new Convert({
      newline: true,
     })
      setOutput(convert.toHtml(outputError));

    setStatus("Displayed at " + datetimeApi.getTimeToString());
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
