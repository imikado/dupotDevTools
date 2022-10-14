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

export default function MyTemplateFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  //command
  const inputFile = featureApi.getTempFilePath("input");
  const outputFile = featureApi.getTempFilePath("output");
  //command line to change
  const command =
    settingsObj.binaryPath + " -o " + outputFile + " " + inputFile;

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

  const launch = () => {
    if (!systemApi.doesBinaryExist(settingsObj.binaryPath)) {
      return window.alert(
        settingsObj.binaryPath + " is missing, please install it before"
      );
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

      <Box component="form" noValidate autoComplete="off">
        <div style={{ textAlign: "right" }}>
          <Button variant="default" onClick={handleShow}>
            Settings
          </Button>
        </div>

        <FormControl fullWidth>
          <TextField
            label="Input"
            multiline
            rows={4}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            helperText="JSON to format"
          />

          <TextField
            label="Output"
            multiline
            rows={9}
            value={output}
            readOnly
            helperText={command}
          />

          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button variant="contained" onClick={launch}>
                Launch
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
