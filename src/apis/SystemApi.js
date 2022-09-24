const nodejs=window.nodejs;

export default class SystemApi{


    readFilePath(filename_){
        return nodejs.readFilePath(filename_);
    }
    writeFilePath(filename_,input_){
        return nodejs.writeFilePath(filename_, input_);
    }
    launchCommand(command_){
        return nodejs.launchCommand(command_);
    }


    

};

