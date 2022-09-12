const { readdirSync, statSync , writeFileSync,readFileSync,unlinkSync } = require('fs')
const {  join } = require('path');

const dirs = p => readdirSync(p).filter(f => statSync(join(p, f)).isDirectory())
const files = p => readdirSync(p).filter(f => !statSync(join(p, f)).isDirectory())

const featureDirectory='../src/features';

let routingContent=readFileSync(join('templates','routing.js'),'utf8');


let templateImportList=[];
let templateComponentList=[];

const regexFeatureFile=/Feature/;

let sectionDirList= dirs(featureDirectory);
for(sectionDirKey in sectionDirList){

    let sectionLoop=sectionDirList[sectionDirKey];

    let featureDirList=dirs(featureDirectory+'/'+sectionLoop);
    for(featureDirKey in featureDirList){

        let featureLoop=featureDirList[featureDirKey];

        let filenameLoop=join(featureDirectory,sectionLoop,featureLoop,'card.json');
        let cardObj=JSON.parse(readFileSync( filenameLoop,'utf8'));
        cardObj.section={
            path:sectionLoop
        };
        cardObj.path=featureLoop;

        writeFileSync(  filenameLoop, JSON.stringify(cardObj, null,4), err => {
            if (err) {
              console.error(err);
            }
            console.log('Write ');
        });

        let featureClassFile=null;

        let fileList=files(join(featureDirectory,sectionLoop,featureLoop));
        console.log(featureLoop);
        for(fileKey in fileList){
            let fileLoop=fileList[fileKey];
            if(fileLoop.match(regexFeatureFile)){
                featureClassFile=fileLoop;
            }
        }

        if(!featureClassFile){
            console.error('Feature class file not found for feature: '+featureLoop);
        }

        let featureClass=featureClassFile.slice(0,-3);

        templateImportList.push('import '+featureClass+' from \'../features/'+sectionLoop+'/'+featureLoop+'/'+featureClass+'\'; ' );

        templateComponentList.push('<Route path="/section/'+sectionLoop+'/'+featureLoop+'/" element={<'+featureClass+' />} />  ');

        //console.log(cardObj);
    }

}

routingContent=routingContent.replace('//#IMPORT_FEATURES_LIST#',templateImportList.join("\n"));
routingContent=routingContent.replace('<ROUTES_LOOP></ROUTES_LOOP>',templateComponentList.join("\n"));

console.log(routingContent);

console.log(templateImportList);
console.log(templateComponentList);

let routingFile=join('..','src','conf','Routing.js');
writeFileSync(  routingFile, routingContent, err => {
    if (err) {
      console.error(err);
    }
    console.log('Write ');
});



