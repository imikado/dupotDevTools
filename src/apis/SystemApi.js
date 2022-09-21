const nodejs=window.nodejs;

export default class SystemApi{

    loadCard(card){
        if(!card.path){
            throw new Error('Missing field path in your card');
        }
        if(!card.section || !card.section.path){
            throw new Error('Missing field section.path in your card');
        }
        this.card=card;
    }

    getTempFilePath(filename_){
        return nodejs.getTempFilePath( this.card.path+'.'+filename_);
    }

    readTempFile(filename_){
        return nodejs.readTempFile(filename_);
    }
    writeTempFile(filename_,input_){
        return nodejs.writeTempFile(filename_, input_);
    }
    launchCommand(command_){
        return nodejs.launchCommand(command_);
    }
    getTimeToString(){
        var date=new Date();
        
        var hours=date.getHours();
        var minutes=date.getMinutes();

        if(hours < 10){
            hours='0'+hours;
        }
        if(minutes < 10){
            minutes='0'+minutes;
        }
        return hours+'+minutes';
    }

    readJsonSettings(){
        return nodejs.getJsonFromFeatureFile(this.getSectionPath(),this.getPath(),'settings.json');
    }
    saveJsonSettings(settingsObj_){
        
        return nodejs.saveFeatureJsonFile(this.getSectionPath(),this.getPath(),'settings.json', settingsObj_);
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

    

};

