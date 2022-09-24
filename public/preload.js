const { contextBridge } = require("electron");

const {
  readdirSync,
  statSync,
  writeFileSync,
  readFileSync,
  unlinkSync,
} = require("fs");
const { join } = require("path");
const { execSync } = require("child_process");

var crypto = require("crypto");

const joinPathList = (pathList_) => pathList_.join("/");
const getDirectoryListWithPath = (p) =>
  readdirSync(p).filter((f) => statSync(join(p, f)).isDirectory());
const readFileWithPath = (path_) => readFileSync(path_, "utf8");
const writeFileWithPath = (path_, content_) => {
  writeFileSync(path_, content_, (err) => {
    if (err) {
      console.error(err);
    }
  });
};

const tempDirectory = "/tmp";

contextBridge.exposeInMainWorld("nodejs", {


  getFilePathWithPathList: (pathList) =>{

    return "file:///" + joinPathList(pathList);
  },
 
  getJoinPathList: (pathList) => {
    return joinPathList(pathList);
  },

  getDirectoryListWithPathList: (pathList_) => {
    return getDirectoryListWithPath(joinPathList(pathList_));
  },

  writeFileWithPathList: (pathList_, content_) => {
    writeFileWithPath(joinPathList(pathList_), content_);
  },
  readFileWithPathList: (pathList_) => {
    return readFileWithPath(joinPathList(pathList_));
  },
  removeFileWithPathList: (pathList_) => {
    return unlinkSync(joinPathList(pathList_));
  },

  getTempPathList: () => {
    return [tempDirectory];
  },
  getFeaturePathList: () => {
    return [__dirname, "..", "src", "features"];
  },

  //feature

  getJsonWithPathList: (pathList_) => {
    return JSON.parse(readFileWithPath(joinPathList(pathList_)));
  },
  saveJsonWithPathList: (pathList_, obj_) => {
    writeFileWithPath(joinPathList(pathList_), JSON.stringify(obj_));
  },

  launchCommand: (command_) => {
    try {
      console.log(command_);
      let output = execSync(command_).toString();
      return output;
    } catch (e) {
      if (e.status != 0) {
        return "Error: " + e.message;
      }
      return "Error uncatched";
    }
  },
});
