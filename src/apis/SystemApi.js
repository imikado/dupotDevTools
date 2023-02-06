const nodejs = window.nodejs;

export default class SystemApi {
  readFilePath(filename_) {
    return nodejs.readFileWithPathList([filename_]);
  }
  writeFilePath(filename_, input_) {
    return nodejs.writeFileWithPathList([filename_], input_);
  }
  launchCommand(command_) {
    return nodejs.launchCommand(command_);
  }

  doesBinaryExist(binary_) {
    let output = nodejs.launchCommand("which " + binary_);
    if (output.substr(0, 5) !== "Error") {
      return true;
    }
    return false;
  }

  jwtDecode(jwtToken) {
    return nodejs.jwtDecode(jwtToken);
  }

  base64Encode(base64ToEncode) {
    return nodejs.base64Encode(base64ToEncode);
  }

  base64Decode(base64Encoded) {
    return nodejs.base64Decode(base64Encoded);
  }
}
