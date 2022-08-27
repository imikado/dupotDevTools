
export default {

  
    'getSectionsList':()=>{
        return window.main.getDirectoryList(window.main.join(['..','src','features']));
    },

    'getFeaturesListInSection':(section)=>{
        return window.main.getDirectoryList(window.main.join(['..','src','features',section]));

    },

    'getTempFilePath':(filename_)=>{
        return window.main.getTempFilePath(filename_);
    },
    'readTempFile':(filename_)=>{
        return window.main.readTempFile(filename_);
    },
    'writeTempFile':(filename_,input_)=>{
        return window.main.writeTempFile(filename_, input_);
    },
    'launchCommand':(command_)=>{
        return window.main.launchCommand(command_);
    },
    'getTimeToString':()=>{
        var date=new Date();
        
        var hours=date.getHours();
        var minutes=date.getMinutes();

        if(hours < 10){
            hours='0'+hours;
        }
        if(minutes < 10){
            minutes='0'+minutes;
        }
        return hours+':'+minutes;
    }

    

}

