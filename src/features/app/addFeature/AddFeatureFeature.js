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
import Errors from "./Errors";

const appApi = new AppApi();
const featureApi = new FeatureApi();
featureApi.loadCard(card)

const datetimeApi = new DatetimeApi();

export default function AddFeatureFeature() {
  const [name, setName] = useState("");
  const [section, setSection] = useState("");
  const [template, setTemplate] = useState("");

  const [confirmationOpen, setConfirmationOpened] =useState(false);
  const [errorsOpen, setErrorsOpened] =useState(false);

  const [createError,setCreateError]=useState("");

  const [status, setStatus] = useState();

  const sectionList = appApi.getSectionsList();

  const createFeatureName=(feature)=>{
    return feature.substring(0,1).toUpperCase() + feature.substring(1)+'Feature';
  }

  const askCreate =() =>{

    if(appApi.existFeatureInSection(name,section)){
      setCreateError('Feature '+name+' already exist !');
      setErrorsOpened(true);
      return;
    }
    setConfirmationOpened(true);
  }

  const create = () => {
    setConfirmationOpened(false);

    let featureJsName=createFeatureName(name);

    let templateContent=featureApi.readFileWithPathList(['templates',template] ,'utf8');
    let featureContent=templateContent.replace('#TemplateFeature#',featureJsName);

    let featureJsNameFile=featureJsName+'.js';

    appApi.createFeatureInSection(name,section);

    appApi.writeFileInFeatureAndSection(featureJsNameFile,featureContent,name,section)

    let iconPathList=featureApi.getCurrentPathList();
    iconPathList.push('assets');
    iconPathList.push('icon.png');

    let newFeaturePathList=appApi.getFeaturePathList();
    newFeaturePathList.push(section);
    newFeaturePathList.push(name);
    newFeaturePathList.push('icon.png');

    appApi.copyFileFromPathListToPathList(iconPathList,newFeaturePathList);

    setStatus("Feature ["+featureJsName+"] created in section [" +section+ "] at " + datetimeApi.getTimeToString());
  };

  const isValid = () =>{

    if(name!='' && section!='' && template!='' ){
      return true;
    }
    return false;
  };

  return (
    <>
      <Confirmation title="Do you confirm ?" open={confirmationOpen} handleClose={() => setConfirmationOpened(false)} handleAccept={create} />
      <Errors title="Error when try to create" description={createError} open={errorsOpen}  handleClose={() => setErrorsOpened(false) } />

      <Box component="form" noValidate autoComplete="off">
        <FormControl fullWidth sx={{ marginLeft:0}}>
          <TextField
            label="Name"
            required
            value={name}
            onChange={(e) => setName(e.target.value)}
            helperText="Name of feature"
          />
        </FormControl>
        <FormControl fullWidth sx={{ marginLeft:1, marginTop:2}}>
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
              <MenuItem key={sectionLoop} value={sectionLoop}>{sectionLoop}</MenuItem>
            ))}
          </Select>
        </FormControl>
        <FormControl fullWidth sx={{ marginLeft:1,marginTop:2}}>
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
              value="nodejs"
              control={<Radio />}
              label="Nodejs"
            />
            <FormControlLabel
              value="command"
              control={<Radio />}
              label="Command line"
            />
          </RadioGroup>
        </FormControl>
        <FormControl fullWidth>
          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button disabled={!isValid()} variant="contained" onClick={askCreate}>
                Create
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
