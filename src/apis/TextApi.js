import YAML from 'yaml'
import { format as SqlFormat } from 'sql-formatter';

const nodejs=window.nodejs;

export default class TextApi{



    hash(string_,type_){
        return nodejs.hash(string_,type_)
    }

    jsonIndent(jsonContent_){
        try{
            return nodejs.jsonIndent(jsonContent_)
        }catch(e){
            return 'Error:'+e.message;   
        }
    }
    
    htmlEncode(text_){ 
        return nodejs.htmlEncode(text_) 
    }
    htmlDecode(text_){
        return nodejs.htmlDecode(text_) 
    }
   
    convertJsonToYaml(text_){ 
        return YAML.stringify(JSON.parse(text_)) 
    }
    convertYamlToJson(text_){ 
        return JSON.stringify( YAML.parse(text_)) 
    }
   
    
    xmlFormat(xmlContent_){
        try{
            return nodejs.xmlFormat(xmlContent_)
        }catch(e){
            return 'Error:'+e.message;   
        }
    }
    sqlFormat(sqlContent_){
        try{
            
            return SqlFormat(sqlContent_,{
                language: 'spark',
                tabWidth: 2,
                keywordCase: 'upper',
                linesBetweenQueries: 2,
              });
        }catch(e){
            return 'Error:'+e.message;   
        }
    }


    
    

};

