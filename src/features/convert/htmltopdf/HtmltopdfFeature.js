import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
} from "@mui/material";
import React, { useState,useEffect } from "react";

import FeatureApi from "../../../apis/FeatureApi";
import DatetimeApi from "../../../apis/DatetimeApi";
import SystemApi from "../../../apis/SystemApi";

import Settings from "./Settings";
import card from "./card.json";

const datetimeApi = new DatetimeApi();
const systemApi = new SystemApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card);

export default function HtmltopdfFeature() {
  const [input, setInput] = useState("");
  const [genId, setGenId] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  //command
  const inputFile = featureApi.getTempFilePath("input")+".html";
  const outputFile = featureApi.getTempFilePath("output")+".pdf";
  //command line to change
  const command =
    settingsObj.binaryPath + " --load-error-handling ignore " + inputFile + " " + outputFile;

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

  const reset= ()=>{
    setGenId("");
    setStatus("");
  }

  const launch = () => {
    systemApi.writeFilePath(inputFile, input);

    systemApi.launchCommand(command);

    setGenId(Math.random().toString())
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
            onFocus={reset}
            helperText="HTML to convert"
          />

        {input!="" && genId &&
         <iframe style={{height:350,border:1,backgroundColor:"#222"}} src={"file://"+outputFile+"?&id="+genId}></iframe>
        }
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
