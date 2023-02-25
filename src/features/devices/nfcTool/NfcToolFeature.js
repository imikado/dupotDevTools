import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  Tab,
  TableCell,
  Tabs,
  TextField,
} from "@mui/material";
import React, { useState, useEffect } from "react";

import FeatureApi from "../../../apis/FeatureApi";
import DatetimeApi from "../../../apis/DatetimeApi";
import SystemApi from "../../../apis/SystemApi";

import Settings from "./Settings";
import card from "./card.json";
import { json } from "react-router-dom";

const datetimeApi = new DatetimeApi();
const systemApi = new SystemApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card);

const decoder = new TextDecoder("utf-8");

export default function NfcToolFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [tabSelected, selectTab] = useState("");

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  const [cardPresent, setCardPresent] = useState(false);

  //command
  const inputFile = featureApi.getTempFilePath("input");
  const outputFile = featureApi.getTempFilePath("output");
  //command line to change
  const command = settingsObj.binaryPath; // + " " + inputFile;

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

  const getUid = () => {
    let outputCommand = getCommandOutput("getuid");
    let jsonObj = JSON.parse(outputCommand);
    if (jsonObj) {
      return jsonObj.getuid;
    }

    return "";
  };

  const getInfo = () => {
    return JSON.stringify(JSON.parse(getCommandOutput("info")), null, 4);
  };

  const getDetail = () => {
    let outputCommand = getCommandOutput("read");

    //return outputCommand;

    let jsonObj = JSON.parse(outputCommand);
    if (jsonObj) {
      return jsonObj.data;
    }
  };

  const hasInfo = () => {
    let uid = getUid();
    if (uid.length > 2) {
      return true;
    }
    return false;
  };

  const getCommandOutput = (args) => {
    return systemApi.launchCommand(settingsObj.binaryPath + " " + args);
  };

  const isTabSelected = (tabName) => {
    if (tabName === tabSelected) {
      return true;
    }
    return false;
  };

  const displayTabInfo = () => {
    selectTab("info");
    setCardPresent(true);

    let outputCommand = getInfo();

    setOutput(outputCommand);
  };

  const displayTabDetail = () => {
    selectTab("detail");
    setCardPresent(true);

    setOutput(getDetail());
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

      <ButtonGroup
        variant="contained"
        aria-label="outlined primary button group"
      >
        <Button
          onClick={() => displayTabInfo()}
          variant={isTabSelected("info") ? "contained" : "outlined"}
        >
          Info
        </Button>
        <Button
          onClick={() => displayTabDetail()}
          variant={isTabSelected("detail") ? "contained" : "outlined"}
        >
          Details
        </Button>
        <Button
          variant={isTabSelected("edit") ? "contained" : "outlined"}
          disabled
        >
          Edit{" "}
        </Button>
      </ButtonGroup>

      <Box component="form" noValidate autoComplete="off">
        <div style={{ textAlign: "right" }}>
          <Button variant="default" onClick={handleShow}>
            Settings
          </Button>
        </div>

        {cardPresent ? (
          <FormControl fullWidth>
            <TextField
              label="Output"
              multiline
              rows={9}
              value={output}
              readOnly
              helperText={command}
            />

            <p>{status}</p>
          </FormControl>
        ) : null}
      </Box>
    </>
  );
}
