import React, { useState } from "react";
import DatetimeApi from "../../../apis/DatetimeApi";
import YAML from "yaml";
import { Box } from "@mui/system";
import { Button, ButtonGroup, FormControl, TextField } from "@mui/material";

const datetimeApi = new DatetimeApi();

export default function YamlToJsonFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convertToYaml = () => {
    let yamlConverted = YAML.stringify(JSON.parse(input));
    setOutput(yamlConverted);

    setStatus("Converted at " + datetimeApi.getTimeToString());
  };
  const convertToJson = () => {
    let jsonConverted = JSON.stringify(YAML.parse(input));
    setOutput(jsonConverted);

    setStatus("Converted at " + datetimeApi.getTimeToString());
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
            onChange={(e) => setInput(e.target.value)}
            helperText="YAML/JSON to convert to JSON/YAML"
          />

          <TextField
            label="Output"
            multiline
            rows={9}
            value={output}
            InputProps={{
              readOnly: true,
            }}
          />

          <p>{status}</p>

          <div style={{ textAlign: "center" }}>
            <ButtonGroup>
              <Button disabled={input==''} variant="contained" onClick={convertToYaml}>
                Convert to YAML
              </Button>

              <Button disabled={input==''} variant="contained" onClick={convertToJson}>
                Convert to JSON
              </Button>
            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
}
