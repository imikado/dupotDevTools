const nodejs=window.nodejs;

export default class SystemApi{


    readFilePath(filename_){
        return nodejs.readFileWithPathList([filename_]);
    }
    writeFilePath(filename_,input_){
        return nodejs.writeFileWithPathList([filename_], input_);
    }
    launchCommand(command_){
        return nodejs.launchCommand(command_);
    }


    

};

