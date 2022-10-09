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
  const [description, setDescription] = useState("");
  const [binarypath, setBinarypath] = useState("");

  const [confirmationOpen, setConfirmationOpened] = useState(false);
  const [popupOpen, setPopupOpened] = useState(false);

  const [popupTitle, setPopupTitle] = useState("");
  const [popupMessage, setPopupMessage] = useState("");

  const [confirmationTitle, setConfirmationTitle] = useState("");

  const [status, setStatus] = useState();

  const sectionList = appApi.getSectionsList();

  const createFeatureName = (feature) => {
    return (
      feature.substring(0, 1).toUpperCase() + feature.substring(1) + "Feature"
    );
  };

  const openPopup = (title_, message_) => {
    setPopupTitle(title_);
    setPopupMessage(message_);
    setPopupOpened(true);
  };

  const openConfirmation=(title_) =>{
    setConfirmationTitle(title_);
    setConfirmationOpened(true);
  };

  const askCreate = () => {
    if (appApi.existFeatureInSection(name, section)) {
      openPopup(
        "Error when try to create",
        "Feature " + name + " already exist !"
      );
      return;
    }
    openConfirmation('Do you confirm create '+name);
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

    let sourcePathList = featureApi.getCurrentPathList();

    let newFeaturePathList = appApi.getFeaturePathList();
    newFeaturePathList.push(section);
    newFeaturePathList.push(name);

    //icons
    let fromIconPathList = sourcePathList.slice();
    fromIconPathList.push("assets");
    fromIconPathList.push("icon.png");

    let toIconPathList = newFeaturePathList.slice();
    toIconPathList.push("icon.png");

    appApi.copyFileFromPathListToPathList(fromIconPathList, toIconPathList);

    //card
    let cardTemplateContent = featureApi.readFileWithPathList(
      ["assets", "card.json"],
      "utf8"
    );
    let cardContent = cardTemplateContent.replace(
      "descriptionpattern",
      description
    );
    appApi.writeFileInFeatureAndSection(
      "card.json",
      cardContent,
      name,
      section
    );

    if (template === "commandTemplateFeature.js") {
      console.log("command line");
      //settings
      let settingsJsonTemplateContent = featureApi.readFileWithPathList(
        ["assets", "settings.json"],
        "utf8"
      );
      let settingsJsonContent = settingsJsonTemplateContent.replace(
        "yourbinarypattern",
        binarypath
      );
      appApi.writeFileInFeatureAndSection(
        "settings.json",
        settingsJsonContent,
        name,
        section
      );

      //settings js
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
    if (name !== "" && section !== "" && template !== "") {
      return true;
    }
    return false;
  };

  return (
    <>
      <Confirmation
        title={confirmationTitle}
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
          <TextField
            label="Description"
            required
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            helperText="Description of the feature"
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

        {template === "commandTemplateFeature.js" && (
          <FormControl fullWidth sx={{ marginLeft: 1, marginTop: 2 }}>
            <TextField
              label="Binary path"
              required
              value={binarypath}
              onChange={(e) => setBinarypath(e.target.value)}
              helperText="path of the binary"
            />
          </FormControl>
        )}

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
