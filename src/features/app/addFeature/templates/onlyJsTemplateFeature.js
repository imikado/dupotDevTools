import { Box, Button, ButtonGroup, FormControl, TextField } from "@mui/material";
import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";

const datetimeApi=new DatetimeApi();

export default function #TemplateFeature#() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {

    //your code
    let outputGenerated=input;
    
    setOutput('');
    try{
      setOutput(outputGenerated);
    }catch(e){
      console.log('Error'+e.message);
    }
    setStatus('Generated at '+datetimeApi.getTimeToString());
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
              <Button variant="contained" onClick={convert}>
                Format
              </Button>

            </ButtonGroup>
          </div>
        </FormControl>
      </Box>
    </>
  );
 
}
