const nodejs=window.nodejs;

export default class AppApi {


    loadCard(card){
        if(!card.path){
            throw new Error('Missing field path in your card');
        }
        if(!card.section || !card.section.path){
            throw new Error('Missing field section.path in your card');
        }
        this.card=card;
    }

  
    getSectionsList(){
        return nodejs.getDirectoryList(nodejs.join(['..','src','features']));
    }

    getFeaturesListInSection(section){
        let featureList=[];
        let dirList= nodejs.getDirectoryList(nodejs.join(['..','src','features',section]));
        for(let dirKeyLoop in dirList){
            let dirLoop=dirList[dirKeyLoop];
            let featureLoop=nodejs.getJsonFromFeatureFile(section,dirLoop,'card.json');
            featureLoop.name=dirLoop;

            featureList.push(featureLoop);
        }
        return featureList; 
    }

    getIcon(section,feature){
        return nodejs.getIcon(section,feature);
    }

   
    readFeatureFileWithPathList(pathList){
        let rootPathList= ['..', 'src','features',this.getSectionPath(),this.getPath()];

        return nodejs.readFeatureFileWithPathList(  rootPathList.concat(pathList));
    }


    getSectionPath(){
        if(!this.card){
            throw new Error('Missing card, need loadCard()');

        }
        
        if(!this.card.section.path){
            throw new Error('Missing field section.path in your card');
        }
        return this.card.section.path
    }
    getPath(){
        if(!this.card){
            throw new Error('Missing card, need loadCard()');

        }
        if(!this.card.path){
            throw new Error('Missing field path in your card');
        }
        
        return this.card.path
    }


    getTempFilePath(filename_){
        return nodejs.getTempFilePath( this.card.path+'.'+filename_);
    }

    readJsonSettings(){
        return nodejs.getJsonFromFeatureFile(this.getSectionPath(),this.getPath(),'settings.json');
    }
    saveJsonSettings(settingsObj_){
        
        return nodejs.saveFeatureJsonFile(this.getSectionPath(),this.getPath(),'settings.json', settingsObj_);
    }
    

};

