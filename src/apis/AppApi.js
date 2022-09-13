const nodejs=window.nodejs;

export default class AppApi {

  
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

   
    
    

};

