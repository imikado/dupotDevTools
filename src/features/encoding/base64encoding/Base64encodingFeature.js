import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
} from "@mui/material";
import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";

import SystemApi from "../../../apis/SystemApi";

const datetimeApi = new DatetimeApi();
const systemApi = new SystemApi();

export default function Base64encodingFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const encode = () => {
    //your code
    let outputGenerated = systemApi.base64Encode(input);

    setOutput("");
    try {
      setOutput(outputGenerated);
    } catch (e) {
      console.log("Error" + e.message);
    }
    setStatus("Encoded at " + datetimeApi.getTimeToString());
  };

  const decode = () => {
    //your code
    let outputGenerated = systemApi.base64Decode(input);

    setOutput("");
    try {
      setOutput(outputGenerated);
    } catch (e) {
      console.log("Error" + e.message);
    }
    setStatus("Decoded at " + datetimeApi.getTimeToString());
  };

  return (
    <>
      <Box component="form" noValidate autoComplete="off">
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
          />

          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button variant="contained" onClick={encode}>
                Encode
              </Button>
              <Button variant="contained" onClick={decode}>
                Decode
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
