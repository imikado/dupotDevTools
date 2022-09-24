const { contextBridge  } = require('electron')

const { readdirSync, statSync , writeFileSync,readFileSync,unlinkSync } = require('fs')
const {  join } = require('path')
const { execSync } = require("child_process");

var crypto = require('crypto')


const dirs = p => readdirSync(p).filter(f => statSync(join(p, f)).isDirectory())

const tempDirectory='/tmp';

contextBridge.exposeInMainWorld('nodejs', {

  getIcon: (section,feature) =>{

     return 'file:///'+__dirname+'/'+join('..','src','features',section,feature,'icon.png')
  },

  join: (pathList) => {

    pathList.unshift(__dirname);

    return pathList.join('/');
  }, 

  getDirectoryList: (path_) => dirs(path_),

  writeFilePath:(filename_,content_) => {
    writeFileSync(  filename_, content_, err => {
      if (err) {
        console.error(err);
      }
      // file written successfully
    });
  },
  readFeatureFileWithPathList:(pathList_) => {
    return readFileSync( join(__dirname, pathList_),'utf8');
  },
  readFileWithPathList:(pathList_) => {
    return readFileSync( join( pathList_ ),'utf8');
  },
  removeFileWithPathList:(filename_)=>{
    return unlinkSync(filename_);
  },

  readFilePath:(path_) => {
    return readFileSync( path_,'utf8');
  },

  getTempFilePath: (filename_) => {
    return join(tempDirectory,filename_);
  },

  //feature

  getJsonFromFeatureFile:(section_,featureDir_,jsonFilename_)=>{
    console.log(join(__dirname,'..','src','features',section_,featureDir_,jsonFilename_));
    return  JSON.parse(readFileSync( join(__dirname,'..','src','features',section_,featureDir_,jsonFilename_)));
  },
  saveFeatureJsonFile:(section_,featureDir_,jsonFilename_,obj_)=>{
    
    let content=JSON.stringify(obj_);

    writeFileSync( join(__dirname,'..','src','features',section_,featureDir_,jsonFilename_), content, err => {
      if (err) {
        console.error(err);
      }
      // file written successfully
    });

  },

  

  launchCommand: (command_)  => {

    try{
      console.log(command_);
      let output= execSync( command_).toString();
      return output;
    }catch(e){
      if( e.status!=0){
        return 'Error: '+e.message;  
      }
      return 'Error uncatched';
    }
  }
 
  
})