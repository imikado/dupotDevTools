import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
} from "@mui/material";
import React, { useState, useEffect } from "react";

import FeatureApi from "../../../apis/FeatureApi";
import DatetimeApi from "../../../apis/DatetimeApi";
import SystemApi from "../../../apis/SystemApi";

import Settings from "./Settings";
import card from "./card.json";

const datetimeApi = new DatetimeApi();
const systemApi = new SystemApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card);

export default function GraphvizFeature() {
  const [input, setInput] = useState("digraph {  A -> {B C}     } ");
  const [output, setOutput] = useState("");

  const [genId, setGenId] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  //command
  const inputFile = featureApi.getTempFilePath("dot");
  const outputFile = featureApi.getTempFilePath("svg");
  //command line to change
  const command = "dot -Tsvg " + inputFile + " > " + outputFile;

  //settings
  const [show, setShow] = useState(false);
  const handleShow = () => setShow(true);
  const handleClose = () => setShow(false);

  const saveSettings = (settingsObj) => {
    featureApi.saveJsonSettings(settingsObj);
  };

  useEffect(() => {
    setSettingsObj(featureApi.readJsonSettings());
  }, []);

  const reset = () => {
    setGenId("");
    setStatus("");
  };

  const goToDoc= () => {
    window.open("https://graphviz.org/doc/info/lang.html","_blank");
  }

  const launch = () => {
    if (!systemApi.doesBinaryExist(settingsObj.binaryPath)) {
      return window.alert("graphviz is missing, please install it before");
    }
    //your code
    systemApi.writeFilePath(inputFile, input);

    let outputError = systemApi.launchCommand(command);

    var outputConverted = systemApi.readFilePath(outputFile);

    if (outputError) {
      setOutput(outputError);
    } else {
      setOutput(outputConverted);
    }

    setGenId(Math.random().toString());

    setStatus("Generated at " + datetimeApi.getTimeToString());
  };

  return (
    <>
      <Settings
        show={show}
        handleClose={handleClose}
        handleSave={saveSettings}
        settingsObj={settingsObj}
        handleSetSettingsObj={setSettingsObj}
      />
      <div style={{ textAlign: "right" }}>

      <Button variant="default" onClick={goToDoc}>Doc</Button>

        <Button variant="default" onClick={handleShow}>
          Settings
        </Button>
      </div>

      <Box component="form" noValidate autoComplete="off">
        <FormControl fullWidth>
          <TextField
            label="Input"
            multiline
            rows={4}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            helperText="JSON to format"
          />
          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button variant="contained" onClick={launch}>
                Launch
              </Button>
            </ButtonGroup>
          </div>

          <img src={"file://" + outputFile + "?randomid=" + genId} />

          <p>{status}</p>
        </FormControl>
      </Box>
    </>
  );
}
