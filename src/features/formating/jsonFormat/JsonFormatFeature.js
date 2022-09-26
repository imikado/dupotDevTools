import { Box, Button, ButtonGroup, FormControl, TextField } from "@mui/material";
import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";

const datetimeApi=new DatetimeApi();

export default function JsonFormatFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {

    var outputConverted=JSON.stringify( JSON.parse(input), null,4)
    
    setOutput('');
    try{
      setOutput(outputConverted);
    }catch(e){
      console.log('Error'+e.message);
    }
    setStatus('Converted at '+datetimeApi.getTimeToString());
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
              <Button disabled={input==''} variant="contained" onClick={convert}>
                Format
              </Button>

            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
 
}
