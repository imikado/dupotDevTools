import { Button, FormControl, Modal, TextField, Typography } from "@mui/material";
import { Box } from "@mui/system";
import React from "react";

const style = {
  position: 'absolute',
  top: '50%',
  left: '50%',
  transform: 'translate(-50%, -50%)',
  width: 400,
  bgcolor: 'white',
  border: '2px solid #000',
  boxShadow: 24,
  p: 4,
};

export default function Settings(props) {
  const handleClose = () => props.handleClose();

  const save = ()=>{
    props.handleSave(props.settingsObj);

    handleClose();
  }

  return (
    <div>
    <Modal open={props.show} onClose={handleClose}  aria-labelledby="modal-modal-title"
    aria-describedby="modal-modal-description">
      
      <Box sx={style}>
          <Typography id="modal-modal-title" variant="h6" component="h2">
            Settings
          </Typography>
          
          <FormControl fullWidth sx={{ m: 1 }}>

            <TextField 
            label="Binary path"
              value={props.settingsObj.binaryPath}
              onChange={(e) => {
                props.handleSetSettingsObj({binaryPath:e.target.value})
              }}
              placeholder="Enter text to hash"
            />
         
        <Button variant="contained" onClick={ save}>
          Save Changes
        </Button>
        </FormControl>
        </Box>
       </Modal>
       </div>
  );
}
