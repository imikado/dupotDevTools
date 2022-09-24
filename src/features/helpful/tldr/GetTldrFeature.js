import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
  Card,
  CardContent
} from "@mui/material";
import React, { useEffect, useState } from "react";

import Settings from "./Settings";
import card from "./card.json";

import DatetimeApi from "../../../apis/DatetimeApi";
import AppApi from "../../../apis/AppApi";
import SystemApi from "../../../apis/SystemApi";

const systemApi = new SystemApi();

const appApi = new AppApi();
appApi.loadCard(card);

const datetimeApi = new DatetimeApi();

const Convert = require("ansi-to-html");

export default function GetTldrFeature() {
  const section = "helpful";
  const feature = "tldr";

  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  //command
  const command = settingsObj.binaryPath;

  //settings
  const [show, setShow] = useState(false);
  const handleShow = () => setShow(true);
  const handleClose = () => setShow(false);

  const saveSettings = (settingsObj) => {
    appApi.saveJsonSettings(settingsObj);
  };

  useEffect(() => {
    setSettingsObj(appApi.readJsonSettings());
  }, []);

  const convert = () => {
    let outputError = systemApi.launchCommand(command + " " + input);

    var convert = new Convert({
      newline: true,
    });
    setOutput(convert.toHtml(outputError));

    setStatus("Displayed at " + datetimeApi.getTimeToString());
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
            
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder="Enter text to convert"
            helperText="Html to convert to text"
          />

          
          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button variant="contained" onClick={() => convert()}>
                Search
              </Button>
            </ButtonGroup>
          </div>

          <Card >
          <CardContent dangerouslySetInnerHTML={{ __html: output }}>
          </CardContent>
          </Card>
          <p>{status}</p>

        </FormControl>
      </Box>
    </>
  );
}
