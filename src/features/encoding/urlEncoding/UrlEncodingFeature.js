import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
} from "@mui/material";
import React, { useState } from "react";
import DatetimeApi from "../../../apis/DatetimeApi";

const datetimeApi = new DatetimeApi();

export default function UrlEncodingFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const encode = () => {
    setOutput(encodeURI(input));

    setStatus("Encoding at " + datetimeApi.getTimeToString());
  };
  const decode = () => {
    setOutput(decodeURI(input));

    setStatus("Decoding at " + datetimeApi.getTimeToString());
  };

  const resetOutput = () => {
    setOutput("");
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
            onChange={(e) => {
              setInput(e.target.value);
              resetOutput();
            }}
            placeholder="Enter text to encode"
            helperText="String you want to encode/decode from/to URL"
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
