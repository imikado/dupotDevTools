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

const datetimeApi = new DatetimeApi();
const systemApi = new SystemApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card);

export default function NfcToolFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [tabSelected, selectTab] = useState("");

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

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

    if (hasInfo()) {
      displayTabInfo();
    }
  }, [settingsObj.binaryPath]);

  const getUid = () => {
    let outputCommand = getCommandOutput("getuid");
    if (outputCommand.includes("getuid")) {
      let outputCommandList = outputCommand.split("\n");
      for (let key in outputCommandList) {
        let lineLoop = outputCommandList[key];
        if (lineLoop.includes("getuid")) {
          return lineLoop;
        }
      }
    }
    return "";
  };

  const getInfo = () => {
    return getCommandOutput("info");
  };

  const getDetail = () => {
    let outputCommand = "";
    for (let i = 0; i < 16; i++) {
      outputCommand += extractBlockTextFromOutput(
        getCommandOutput("read " + i)
      );
    }
    return outputCommand;
  };

  const extractBlockTextFromOutput = (outputCommandToExtract) => {
    let extractText = "";

    let extractBlockTextFromOutputList = outputCommandToExtract.split("\n");
    for (let keyLoop in extractBlockTextFromOutputList) {
      let lineLoop = extractBlockTextFromOutputList[keyLoop];
      if (lineLoop.substr(0, 5) == "block") {
        let detailLineList = lineLoop.split("|");

        extractText += detailLineList[1];
      }
    }
    return extractText;
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

    let outputCommand = getInfo();

    setOutput(outputCommand);
  };

  const displayTabDetail = () => {
    selectTab("detail");

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
      </Box>
    </>
  );
}
