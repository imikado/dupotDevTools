import { Box, Button, ButtonGroup, FormControl, TextField } from "@mui/material";
import React, { useState } from "react";

import DatetimeApi from "../../../apis/DatetimeApi";
import { format } from 'sql-formatter';

const datetimeApi=new DatetimeApi();

export default function SqlFormatFeature() {
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");

  const [status, setStatus] = useState();

  const convert = () => {

    var outputConverted=format(input,{
      language: 'spark',
      tabWidth: 2,
      keywordCase: 'upper',
      linesBetweenQueries: 2,
    })
    
    setOutput('');
    try{
      setOutput(outputConverted);
    }catch(e){
      console.log('Error'+e.message);
    }
    setStatus('Formated at '+datetimeApi.getTimeToString());
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
            helperText="SQL to format"
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
