const { contextBridge  } = require('electron')

const { readdirSync, statSync , writeFileSync,readFileSync,unlinkSync } = require('fs')
const {  join } = require('path')
const { execSync } = require("child_process");

var crypto = require('crypto')
const HTMLDecoderEncoder = require("html-encoder-decoder");


const dirs = p => readdirSync(p).filter(f => statSync(join(p, f)).isDirectory())

const tempDirectory='/tmp';

contextBridge.exposeInMainWorld('main', {

  join: (pathList) => {

    pathList.unshift(__dirname);

    return pathList.join('/');
  }, 

  getDirectoryList: (path_) => dirs(path_),

  writeTempFile:(filename_,content_) => {
    writeFileSync(  filename_, content_, err => {
      if (err) {
        console.error(err);
      }
      // file written successfully
    });
  },
  readTempFile:(filename_) => {
    return readFileSync( filename_,'utf8');
  },
  removeFile:(filename_)=>{
    return unlinkSync(filename_);
  },

  getTempFilePath: (filename_) => {
    return join(tempDirectory,filename_);
  },
  getJsonFromFeatureFile:(section_,featureDir_,jsonFilename_)=>{
    return  JSON.parse(readFileSync( join(__dirname,'..','src','features',section_,featureDir_,jsonFilename_)));
  },


  launchCommand: (command_)  => {

    return execSync( command_).toString();
  

  },

  hash: (string_,type_)=>{
    var shasum = crypto.createHash(type_)
    shasum.update(string_)
    return shasum.digest('hex');
  },

  jsonIndent: (jsonContent_) => {
    return JSON.stringify( JSON.parse(jsonContent_), null,4);
  },

  htmlEncode: (text_) => {
    return HTMLDecoderEncoder.encode(text_);
  },
  htmlDecode: (text_) => {
    return HTMLDecoderEncoder.decode(text_);
  },
  xmlFormat:(text_)=>{
    var format = require('xml-formatter');

    return format(text_);
  }



   
  // we can also expose variables, not just functions
})