import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  FormControlLabel,
  FormLabel,
  InputLabel,
  MenuItem,
  Radio,
  RadioGroup,
  Select,
  TextField,
} from "@mui/material";
import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";
import AppApi from "../../../apis/AppApi";
import FeatureApi from "../../../apis/FeatureApi";
import card from "./card.json";
import Confirmation from "./Confirmation";
import Popup from "./Popup";

const appApi = new AppApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card);

const datetimeApi = new DatetimeApi();

export default function AddFeatureFeature() {
  const [name, setName] = useState("");
  const [section, setSection] = useState("");
  const [template, setTemplate] = useState("");

  const [confirmationOpen, setConfirmationOpened] = useState(false);
  const [popupOpen, setPopupOpened] = useState(false);

  const [popupTitle, setPopupTitle] = useState("");
  const [popupMessage, setPopupMessage] = useState("");

  const [status, setStatus] = useState();

  const sectionList = appApi.getSectionsList();

  const createFeatureName = (feature) => {
    return (
      feature.substring(0, 1).toUpperCase() + feature.substring(1) + "Feature"
    );
  };

  const openPopup = (title_,message_) =>{
    setPopupTitle(title_);
    setPopupMessage(message_);
    setPopupOpened(true);
  }

  const askCreate = () => {
    if (appApi.existFeatureInSection(name, section)) {
      openPopup("Error when try to create","Feature " + name + " already exist !");
      return;
    }
    setConfirmationOpened(true);
  };

  const create = () => {
    setConfirmationOpened(false);

    let featureJsName = createFeatureName(name);

    let templateContent = featureApi.readFileWithPathList(
      ["templates", template],
      "utf8"
    );
    let featureContent = templateContent.replace(
      "MyTemplateFeature",
      featureJsName
    );

    let featureJsNameFile = featureJsName + ".js";

    appApi.createFeatureInSection(name, section);

    appApi.writeFileInFeatureAndSection(
      featureJsNameFile,
      featureContent,
      name,
      section
    );

    let sourcePathList=featureApi.getCurrentPathList();

    let newFeaturePathList = appApi.getFeaturePathList();
    newFeaturePathList.push(section);
    newFeaturePathList.push(name);

    let fromIconPathList = sourcePathList.slice();
    fromIconPathList.push("assets");
    fromIconPathList.push("icon.png");

    let toIconPathList = newFeaturePathList.slice();
    toIconPathList.push("icon.png");

    appApi.copyFileFromPathListToPathList(fromIconPathList, toIconPathList);

    if (template == "commandTemplateFeature.js") {
      console.log('command line');
      let fromSettingsJsonPathList = sourcePathList.slice();
      fromSettingsJsonPathList.push("assets");
      fromSettingsJsonPathList.push("settings.json");

      let toSettingsJsonPathList = newFeaturePathList.slice();
      toSettingsJsonPathList.push("settings.json");

      appApi.copyFileFromPathListToPathList(
        fromSettingsJsonPathList,
        toSettingsJsonPathList
      );

      let fromSettingsJsPathList = sourcePathList.slice();
      fromSettingsJsPathList.push("assets");
      fromSettingsJsPathList.push("Settings.js");

      let toSettingsJsPathList = newFeaturePathList.slice();
      toSettingsJsPathList.push("Settings.js");

      appApi.copyFileFromPathListToPathList(
        fromSettingsJsPathList,
        toSettingsJsPathList
      );
    }

    setStatus(
      "Feature [" +
        featureJsName +
        "] created in section [" +
        section +
        "] at " +
        datetimeApi.getTimeToString()
    );
  };

  const isValid = () => {
    if (name != "" && section != "" && template != "") {
      return true;
    }
    return false;
  };

  return (
    <>
      <Confirmation
        title="Do you confirm ?"
        open={confirmationOpen}
        handleClose={() => setConfirmationOpened(false)}
        handleAccept={create}
      />
      <Popup
        title={popupTitle}
        description={popupMessage}
        open={popupOpen}
        handleClose={() => setPopupOpened(false)}
      />

      <Box component="form" noValidate autoComplete="off">
        <FormControl fullWidth sx={{ marginLeft: 0 }}>
          <TextField
            label="Name"
            required
            value={name}
            onChange={(e) => setName(e.target.value)}
            helperText="Name of feature"
          />
        </FormControl>
        <FormControl fullWidth sx={{ marginLeft: 1, marginTop: 2 }}>
          <InputLabel id="section-select-label">Section</InputLabel>
          <Select
            labelId="section-select-label"
            id="section-select"
            value={section}
            label="Section"
            required
            onChange={(e) => setSection(e.target.value)}
          >
            {sectionList.map((sectionLoop) => (
              <MenuItem key={sectionLoop} value={sectionLoop}>
                {sectionLoop}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
        <FormControl fullWidth sx={{ marginLeft: 1, marginTop: 2 }}>
          <FormLabel id="template-group-label">Template</FormLabel>
          <RadioGroup
            aria-labelledby="template-group-label"
            name="template-group"
            onChange={(e) => setTemplate(e.target.value)}
          >
            <FormControlLabel
              value="onlyJsTemplateFeature.js"
              control={<Radio />}
              label="Only JS"
            />
          
            <FormControlLabel
              value="commandTemplateFeature.js"
              control={<Radio />}
              label="Command line"
            />
          </RadioGroup>
        </FormControl>
        <FormControl fullWidth>
          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button
                disabled={!isValid()}
                variant="contained"
                onClick={askCreate}
              >
                Create
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
