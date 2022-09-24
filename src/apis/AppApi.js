const nodejs = window.nodejs;

export default class AppApi {
  loadCard(card) {
    if (!card.path) {
      throw new Error("Missing field path in your card");
    }
    if (!card.section || !card.section.path) {
      throw new Error("Missing field section.path in your card");
    }
    this.card = card;
  }

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

  getCurrentFeaturePathList() {
    return nodejs
      .getFeaturePathList()
      .concat([this.getSectionPath(), this.getPath()]);
  }

  getTempFilePath(filename_) {
    let tempFilePathList = nodejs
      .getTempPathList()
      .concat([this.card.path + "." + filename_]);
    return nodejs.getJoinPathList(tempFilePathList);
  }

  getIcon(section, feature) {
    let iconPathList=nodejs.getFeaturePathList().concat([section,feature,"icon.png"]);
    return nodejs.getFilePathWithPathList(iconPathList);
  }

  getSectionPath() {
    if (!this.card) {
      throw new Error("Missing card, need loadCard()");
    }

    if (!this.card.section.path) {
      throw new Error("Missing field section.path in your card");
    }
    return this.card.section.path;
  }
  getPath() {
    if (!this.card) {
      throw new Error("Missing card, need loadCard()");
    }
    if (!this.card.path) {
      throw new Error("Missing field path in your card");
    }

    return this.card.path;
  }

  readFeatureFileWithPathList(pathList) {
    return nodejs.readFeatureFileWithPathList(
      this.getCurrentFeaturePathList().concat(pathList)
    );
  }

  readJsonSettings() {
    return nodejs.getJsonWithPathList(
      this.getCurrentFeaturePathList().concat(["settings.json"])
    );
  }
  saveJsonSettings(settingsObj_) {
    return nodejs.saveJsonWithPathList(
      this.getCurrentFeaturePathList().concat(["settings.json"]),
      settingsObj_
    );
  }
}
