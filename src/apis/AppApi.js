const nodejs = window.nodejs;

export default class AppApi {
  

  getSectionsList() {
    let featurePathList = nodejs.getFeaturePathList();
    return nodejs.getDirectoryListWithPathList(featurePathList);
  }

  getFeaturesListInSection(section) {
    let featureList = [];
    let sectionPathList = nodejs.getFeaturePathList();
    sectionPathList.push(section);
    let dirList = nodejs.getDirectoryListWithPathList(sectionPathList);
    for (let dirKeyLoop in dirList) {
      let dirLoop = dirList[dirKeyLoop];

      let jsonPathList = sectionPathList.concat([dirLoop, "card.json"]);
      let featureLoop = nodejs.getJsonWithPathList(jsonPathList);
      featureLoop.name = dirLoop;

      featureList.push(featureLoop);
    }
    return featureList;
  }

  getIconForSectionAndFeature(section, feature) {
    let iconPathList=nodejs.getFeaturePathList().concat([section,feature,"icon.png"]);
    return nodejs.getFilePathWithPathList(iconPathList);
  }

}
