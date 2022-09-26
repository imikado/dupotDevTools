import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";
import HTMLDecoderEncoder from "html-encoder-decoder";
import { Box } from "@mui/system";
import { Button, ButtonGroup, FormControl, TextField } from "@mui/material";

const datetimeApi = new DatetimeApi();

export default function HtmlEncodingFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const encode = () => {
    setOutput(HTMLDecoderEncoder.encode(input));

    setStatus("Encoding at " + datetimeApi.getTimeToString());
  };
  const decode = () => {
    setOutput(HTMLDecoderEncoder.decode(input));

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
            required
            rows={4}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              resetOutput();
            }}
            placeholder="Enter text to encode"
            helperText="String you want to encode/decode from/to HTML"
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
              <Button disabled={input==''} variant="contained" onClick={encode}>
                Encode
              </Button>

              <Button disabled={input==''} variant="contained" onClick={decode}>
                Decode
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
