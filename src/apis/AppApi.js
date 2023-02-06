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

  getFeaturesList() {
    let entireFeatureList = [];
    let sectionList = this.getSectionsList();
    for (let sectionKeyLoop in sectionList) {
      let sectionLoop = sectionList[sectionKeyLoop];
      let featureInSectionList = this.getFeaturesListInSection(sectionLoop);
      for (let featureInSectionKeyLoop in featureInSectionList) {
        entireFeatureList.push(featureInSectionList[featureInSectionKeyLoop]);
      }
    }
    console.log(entireFeatureList);
    return entireFeatureList;
  }

  getIconForSectionAndFeature(section, feature) {
    let iconPathList = nodejs
      .getFeaturePathList()
      .concat([section, feature, "icon.png"]);
    return nodejs.getFilePathWithPathList(iconPathList);
  }

  existFeatureInSection(feature, section) {
    let sectionPathList = nodejs.getFeaturePathList();
    sectionPathList.push(section);

    let currentFeatureList =
      nodejs.getDirectoryListWithPathList(sectionPathList);
    console.log(currentFeatureList.indexOf(feature));
    if (currentFeatureList.indexOf(feature) >= 0) {
      return true;
    }
    return false;
  }

  getFeaturePathList() {
    return nodejs.getFeaturePathList();
  }

  createFeatureInSection(feature, section) {
    let sectionPathList = nodejs.getFeaturePathList();
    sectionPathList.push(section);
    sectionPathList.push(feature);

    return nodejs.createDirectoryWithPathList(sectionPathList);
  }

  writeFileInFeatureAndSection(filename, content, feature, section) {
    let pathList = nodejs.getFeaturePathList();
    pathList.push(section);
    pathList.push(feature);
    pathList.push(filename);

    return nodejs.writeFileWithPathList(pathList, content);
  }

  copyFileFromPathListToPathList(fromPathList, toPathList) {
    return nodejs.copyFileFromPathListToPathList(fromPathList, toPathList);
  }
}
