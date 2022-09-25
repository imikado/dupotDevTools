import React, { useEffect, useState } from "react";

import Settings from "./Settings";
import card from "./card.json";
import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
} from "@mui/material";

import FeatureApi from "../../../apis/FeatureApi";
import DatetimeApi from "../../../apis/DatetimeApi";
import SystemApi from "../../../apis/SystemApi";

const datetimeApi = new DatetimeApi();
const systemApi = new SystemApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card);

export default function Html2TextFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  //command
  const inputFile = featureApi.getTempFilePath("input");
  const outputFile = featureApi.getTempFilePath("output");
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

  const convert = () => {
    systemApi.writeFilePath(inputFile, input);

    let outputError = systemApi.launchCommand(command);

    var outputConverted = systemApi.readFilePath(outputFile);

    if (outputError) {
      setOutput(outputError);
    } else {
      setOutput(outputConverted);
    }
    setStatus("Converted at " + datetimeApi.getTimeToString());
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
            placeholder="Enter text to convert"
            helperText="Html to convert to text"
          />

          <TextField
            label="Output"
            multiline
            rows={9}
            value={output}
            InputProps={{
              readOnly: true,
            }}
            helperText={command}
          />

          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button variant="contained" onClick={() => convert()}>
                Convert
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
