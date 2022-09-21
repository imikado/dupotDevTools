import {
  Box,
  Button,
  ButtonGroup,
  FormControl,
  TextField,
} from "@mui/material";
import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";
import crypto from "crypto-js";

const datetimeApi = new DatetimeApi();

export default function HashFeature() {
  const [input, setInput] = useState("");
  const [outputMd5, setOutputMd5] = useState("");
  const [outputSha1, setOutputSha1] = useState("");
  const [outputSha256, setOutputSha256] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {
    setOutputMd5(crypto.MD5(input));
    setOutputSha1(crypto.SHA1(input));
    setOutputSha256(crypto.SHA256(input));

    setStatus("Converted at " + datetimeApi.getTimeToString());
  };

  const resetOutput = () => {
    setOutputMd5("");
    setOutputSha1("");
    console.log("reset");
  };

  return (
    <>
      <Box component="form" noValidate autoComplete="off">
        <FormControl fullWidth>
          <TextField
            label="Input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            helperText="Text to hash"
          />

          <TextField
            label="Md5"
            value={outputMd5}
            InputProps={{
              readOnly: true,
            }}
          />

          <TextField
            label="Sha1"
            value={outputSha1}
            InputProps={{
              readOnly: true,
            }}
          />

          <TextField
            label="Sha256"
            value={outputSha256}
            InputProps={{
              readOnly: true,
            }}
          />

          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button variant="contained" onClick={convert}>
                Hash
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
