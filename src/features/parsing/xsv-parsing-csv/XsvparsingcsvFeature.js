import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  InputLabel,
  MenuItem,
  Select,
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

export default function XsvparsingcsvFeature() {
  const [action, setAction] = useState("");
  const [delimiter, setDelimiter] = useState(";");

  const [option1, setOption1] = useState('');
  const [option2, setOption2] = useState('');
  const [option3, setOption3] = useState('');

  const [option1Value, setOption1Value] = useState("");
  const [option2Value, setOption2Value] = useState("");
  const [option3Value, setOption3Value] = useState("");

  const [option1State, setOption1State] = useState(false);
  const [option2State, setOption2State] = useState(false);
  const [option3State, setOption3State] = useState(false);

  const TYPE_TEXT = "text";
  const TYPE_SELECT = "select";
  const TYPE_YESNO_WITH_TEXT = "yesno_with_text";

  const OPTION_EMPTY = "EMPTY";

  const [commandLaunched, setCommandLaunched] = useState("");

  const [inputFile, setInputFile] = useState("");

  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const [settingsObj, setSettingsObj] = useState({ binaryPath: "aa" });

  const actionList = [
    //"cat",
    "count",
    "fixlengths",
    "flatten",
    'fmt',
    "frequency",
    "headers",
    //'index',
    //"input",
    //"join",
    //"sample",
    //"search",
    "select",
    //"slice",
    "sort",
    //"stats",
    "table",
  ];

  const returnIf=(string_,state_)=>{
    if(state_){
      return string_;
    }
    return '';
  }

  const optionIndexedList = {
    count: {
      command:
        settingsObj.binaryPath +
        " " +
        action +
        " " +
        option1Value +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile,
      options: [
        {
          type: TYPE_SELECT,
          label: "Count headers ?",
          id: "count",
          valuesList: [
            {
              label: "Yes",
              value: "-n",
            },
            {
              label: "No",
              value: OPTION_EMPTY,
            },
          ],
        },
      ],
    },
    fixlengths: {
      command:
        settingsObj.binaryPath +
        " " +
        action +
        " -l " +
        option1Value +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile,
      options: [
        {
          type: TYPE_TEXT,
          value: "",
          label: "Length",
        },
      ],
    },
    flatten: {
      command:
        settingsObj.binaryPath +
        " " +
        action +
        " " +
        returnIf("-c ",option1State) +
        option1Value +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile,
      options: [
        {
          type: TYPE_YESNO_WITH_TEXT,
          value: "-c",
          label: "Condense",
        },
      ],
    },
    fmt:{
      command:
        settingsObj.binaryPath +
        " " +
        action +
        " " +
        option1Value +
        
        option2Value +
        option3Value +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile,
        options: [
          {
            type: TYPE_SELECT,
            label: "use CRLF line ending ?",
            id: "crlf",
            valuesList: [
              {
                label: "Yes",
                value: "--crlf ",
              },
              {
                label: "No",
                value: OPTION_EMPTY,
              },
            ],
          },
          {
            type: TYPE_SELECT,
            label: "use ASCII field/record sep ?",
            id: "ascii",
            valuesList: [
              {
                label: "Yes",
                value: "--ascii ",
              },
              {
                label: "No",
                value: OPTION_EMPTY,
              },
            ],
          },
          {
            type: TYPE_SELECT,
            label: "Put quotes around every value ?",
            id: "quotealways",
            valuesList: [
              {
                label: "Yes",
                value: "--quote-always ",
              },
              {
                label: "No",
                value: OPTION_EMPTY,
              },
            ],
          },
        ],
    },
    headers:{
      command:
      settingsObj.binaryPath +
      " " +
      action +
      " " +
      option1Value +
      
      option2Value +
      ' -d"' +
      delimiter +
      '" ' +
      inputFile,
      options: [
        {
          type: TYPE_SELECT,
          label: "Only show the header names ?",
          id: "onlynames",
          valuesList: [
            {
              label: "Yes",
              value: "--just-names ",
            },
            {
              label: "No",
              value: OPTION_EMPTY,
            },
          ],
        },
        {
          type: TYPE_SELECT,
          label: "Shows the intersection of all headers ?",
          id: "intersection",
          valuesList: [
            {
              label: "Yes",
              value: "--intersect ",
            },
            {
              label: "No",
              value: OPTION_EMPTY,
            },
          ],
        }
      ]
    },
    select: {
      command:
      settingsObj.binaryPath +
      " " +
      action +
      " " +
      option1Value +
      
      ' -d"' +
      delimiter +
      '" ' +
      inputFile,
      options: [
        {
          type: TYPE_TEXT,
          value: "",
          label: "Field to select",
        },
      ],
    },
    sort:{
      command:
        settingsObj.binaryPath +
        " " +
        action +
        " " +
        returnIf("-s ",option1State) +
        option1Value +
        
        option2Value +
        option3Value +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile,
        options: [
          {
            type: TYPE_YESNO_WITH_TEXT,
            value: "-s",
            label: "subset of columns to sort",
          },

          {
            type: TYPE_SELECT,
            label: "compare as numeric value ?",
            id: "numeric",
            valuesList: [
              {
                label: "Yes",
                value: "--numeric ",
              },
              {
                label: "No",
                value: OPTION_EMPTY,
              },
            ],
          },
          {
            type: TYPE_SELECT,
            label: "reverse order ?",
            id: "reverse",
            valuesList: [
              {
                label: "Yes",
                value: "--reverse ",
              },
              {
                label: "No",
                value: OPTION_EMPTY,
              },
            ],
          }
        ],
    },
    table:{
      command:
        settingsObj.binaryPath +
        " " +
        action +
        " " +
        returnIf(" -w",option1State) +
        option1Value +
        
        returnIf(" -p",option2State) +
        option2Value +

        returnIf(" -c",option3State) +
        option3Value +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile,
        options: [
          {
            type: TYPE_YESNO_WITH_TEXT,
            value: "-w",
            label: "minimum width of each column",
          },
          {
            type: TYPE_YESNO_WITH_TEXT,
            value: "-p",
            label: "minimum number of spaces between",
          },
          {
            type: TYPE_YESNO_WITH_TEXT,
            value: "-c",
            label: "Limits the length of each field",
          },

          
        ],
    },
  };

  const delimiterList = [
    {
      value: ";",
      label: "semicolon",
    },
    {
      value: ",",
      label: "comma",
    },
  ];

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
    if (inputFile === "") {
      return window.alert("Need to select input file");
    }
    if (action === "") {
      return window.alert("Need to select an action");
    }

    //command line to change
    let command = optionIndexedList[action].command;

    command = command.replaceAll(OPTION_EMPTY, "");

    /*
    if (action == "select") {
      command =
        settingsObj.binaryPath +
        " " +
        action +
        ' -d"' +
        delimiter +
        '" ' +
        option1 +
        " " +
        inputFile;
    } else if (action == "table") {
      command =
        settingsObj.binaryPath +
        " " +
        action +
        ' -d"' +
        delimiter +
        '" ' +
        inputFile;
    } else if (action == "fixlengths") {
      command =
        settingsObj.binaryPath +
        " " +
        action +
        ' -d"' +
        delimiter +
        '" -l' +
        option1 +
        " " +
        inputFile;
    }*/

    let outputError = systemApi.launchCommand(command);

    setCommandLaunched(command);

    setOutput(outputError);

    setStatus("Generated at " + datetimeApi.getTimeToString());
  };

  const selectAction = (action_) => {
    setAction(action_);

    //setOption1(null);

    if (optionIndexedList[action_]) {
      let i = 1;
      for (let keyLoop in optionIndexedList[action_].options) {
        let inputLoop = optionIndexedList[action_].options[keyLoop];

        if (i == 1) {
          setOption1Value('');
          setOption1(inputLoop);
          setOption1State(false);
        }else if (i == 2) {
          setOption2Value('');
          setOption2(inputLoop);
          setOption2State(false);
        }else if (i == 3) {
          setOption3Value('');
          setOption3(inputLoop);
          setOption3State(false);
        }

        i++;
      }
      for(let j=i;j<4;j++){
        if (j == 1) {
          setOption1Value('');
          setOption1('');
          setOption1State(false);
        }else if (j == 2) {
          setOption2Value('');
          setOption2('');
          setOption2State(false);
        }else if (j == 3) {
          setOption3Value('');
          setOption3('');
          setOption3State(false);
        }
      }
    }
  };

  const selectInput = async () => {
    setInputFile(
      await featureApi.showOpenDialogFile([
        { name: "Csv files", extensions: ["csv"] },
      ])
    );
  };

  const getInputFileBasename = () => {
    if (inputFile) {
      return <> ( {basename(inputFile)} )</>;
    }
    return null;
  };

  const basename = (path) => {
    return path.split("/").reverse()[0];
  };

  const getOptionInput = (
    option_,
    optionValue_,
    optionState_,
    setOptionState_,
    setOptionValue_
  ) => {
    if (!option_) {
      return null;
    }
    if (option_.type === TYPE_TEXT) {
      return (
        <FormControl fullWidth sx={{ marginLeft: 0 }}>
          <TextField
            label={option_.label}
            required
            value={optionValue_}
            onChange={(e) => setOptionValue_(e.target.value)}
            helperText={option_.label}
          />
        </FormControl>
      );
    } else if (option_.type === TYPE_SELECT) {
      return (
        <FormControl sx={{ marginLeft: 1, marginTop: 2 }}>
          <InputLabel id={option_.id + "-select-label"}>
            {option_.label}
          </InputLabel>
          <Select
            labelId={option_.id + "-select-label"}
            id={option_.id + "-select"}
            value={optionValue_}
            label={option_.label}
            required
            onChange={(e) => setOptionValue_(e.target.value)}
          >
            {option_.valuesList.map((valueListLoop) => (
              <MenuItem key={valueListLoop.value} value={valueListLoop.value}>
                {valueListLoop.label}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
      );
    } else if (option_.type === TYPE_YESNO_WITH_TEXT) {
      let yesNoList = [
        {
          value: false,
          label: "No",
        },
        {
          value: true,
          label: "Yes",
        },
      ];

      return (
        <FormControl sx={{ marginLeft: 1, marginTop: 2 }}>
          <InputLabel id={option_.id + "-select-label"}>
            {option_.label}
          </InputLabel>
          <Select
            labelId={option_.id + "-select-label"}
            id={option_.id + "-select"}
            value={optionState_}
            label={option_.label}
            required
            onChange={(e) => setOptionState_(e.target.value)}
          >
            {yesNoList.map((valueListLoop) => (
              <MenuItem key={valueListLoop.value} value={valueListLoop.value}>
                {valueListLoop.label}
              </MenuItem>
            ))}
          </Select>

          {optionState_ && (
            <TextField
              label={option_.label}
              required
              value={optionValue_}
              onChange={(e) => setOptionValue_(e.target.value)}
              helperText={option_.label}
            />
          )}
        </FormControl>
      );
    }

    return null;
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
          <Button onClick={selectInput}>
            Load CSV {getInputFileBasename()}
          </Button>

          <FormControl sx={{ marginLeft: 1, marginTop: 2 }}>
            <InputLabel id="delimiter-select-label">Delimiter</InputLabel>
            <Select
              labelId="delimiter-select-label"
              id="delimiter-select"
              value={delimiter}
              label="Delimiter"
              required
              onChange={(e) => setDelimiter(e.target.value)}
            >
              {delimiterList.map((delimiterLoop) => (
                <MenuItem key={delimiterLoop.value} value={delimiterLoop.value}>
                  {delimiterLoop.label}
                </MenuItem>
              ))}
            </Select>
          </FormControl>

          <FormControl sx={{ marginLeft: 1, marginTop: 2 }}>
            <InputLabel id="action-select-label">Action</InputLabel>
            <Select
              labelId="action-select-label"
              id="action-select"
              value={action}
              label="action"
              required
              onChange={(e) => selectAction(e.target.value)}
            >
              {actionList.map((actionLoop) => (
                <MenuItem key={actionLoop} value={actionLoop}>
                  {actionLoop}
                </MenuItem>
              ))}
            </Select>
          </FormControl>

          {getOptionInput(
            option1,
            option1Value,
            option1State,
            setOption1State,
            setOption1Value
          )}
          {getOptionInput(
            option2,
            option2Value,
            option2State,
            setOption2State,
            setOption2Value
          )}
          {getOptionInput(
            option3,
            option3Value,
            option3State,
            setOption3State,
            setOption3Value
          )}

          <TextField
            label="Output"
            multiline
            rows={9}
            value={output}
            readOnly
            helperText={commandLaunched}
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
