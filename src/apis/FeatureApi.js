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

  getCurrentPathList() {
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

  readFileWithPathList(pathList) {
    return nodejs.readFileWithPathList(
      this.getCurrentPathList().concat(pathList)
    );
  }

  readJsonSettings() {
    return nodejs.getJsonWithPathList(
      this.getCurrentPathList().concat(["settings.json"])
    );
  }
  saveJsonSettings(settingsObj_) {
    return nodejs.saveJsonWithPathList(
      this.getCurrentPathList().concat(["settings.json"]),
      settingsObj_
    );
  }

  async showOpenDialogFile(){

    return await nodejs.showOpenDialog({ properties: ['openFile', 'multiSelections'] });
  }


}
