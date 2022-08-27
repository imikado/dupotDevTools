import YAML from 'yaml'


export default {

  
    'getSectionsList':()=>{
        return window.main.getDirectoryList(window.main.join(['..','src','features']));
    },

    'getFeaturesListInSection':(section)=>{
        let featureList=[];
        let dirList= window.main.getDirectoryList(window.main.join(['..','src','features',section]));
        for(let dirKeyLoop in dirList){
            let dirLoop=dirList[dirKeyLoop];
            let featureLoop=window.main.getJsonFromFeatureFile(section,dirLoop,'card.json');
            featureLoop.name=dirLoop;

            featureList.push(featureLoop);
        }
        return featureList; 
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
    },

    'hash':(string_,type_)=> window.main.hash(string_,type_),

    'jsonIndent':(jsonContent_)=> {
        try{
            return window.main.jsonIndent(jsonContent_)
        }catch(e){
            return 'Error:'+e.message;   
        }
    },
    
    'htmlEncode':(text_)=>window.main.htmlEncode(text_),
    'htmlDecode':(text_)=>window.main.htmlDecode(text_),
   
    'convertJsonToYaml':(text_)=>YAML.stringify(JSON.parse(text_)),
    'convertYamlToJson':(text_)=>JSON.stringify( YAML.parse(text_)),
   
    
    'xmlFormat':(xmlContent_)=> {
        try{
            return window.main.xmlFormat(xmlContent_)
        }catch(e){
            return 'Error:'+e.message;   
        }
    },
    

}

