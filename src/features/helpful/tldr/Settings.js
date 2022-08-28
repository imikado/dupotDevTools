import React, { useEffect, useState } from "react";
import { Form, Modal, Button } from "react-bootstrap";

export default function Settings(props) {
  //const [show, setShow] = useState(false);
  const handleClose = () => props.handleClose();

  //const [binaryPath, setBinaryPath] = useState('');

  /*useEffect(() => {
    setShow(props.show)
    //setBinaryPath(props.settingsObj.binaryPath);
  },[props.show,props.settingsObj]);
  */
  const save = ()=>{
    //let settingsObj=props.settingsObj;
    //settingsObj.binaryPath=binaryPath;
    props.handleSave(props.settingsObj);

    handleClose();
  }

  return (
    <Modal show={props.show} onHide={handleClose}>
      <Modal.Header>
        <Modal.Title>Settings</Modal.Title>
      </Modal.Header>
      <Modal.Body>
        <Form>
          <Form.Group className="mb-3">
            <Form.Label>Binary path</Form.Label>
            <Form.Control
              value={props.settingsObj.binaryPath}
              onChange={(e) => {
                props.handleSetSettingsObj({binaryPath:e.target.value})
                //props.settingsObj.binaryPath=e.target.value;
              }}
              placeholder="Enter text to hash"
            />
          </Form.Group>
        </Form>
      </Modal.Body>
      <Modal.Footer>
        <Button variant="secondary" onClick={handleClose}>
          Close
        </Button>
        <Button variant="primary" onClick={ save}>
          Save Changes
        </Button>
      </Modal.Footer>
    </Modal>
  );
}
