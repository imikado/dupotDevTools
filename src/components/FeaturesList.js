import React from 'react';
import { Button, Card } from 'react-bootstrap';
import { Link, useParams } from 'react-router-dom';

import AppApi from '../apis/AppApi';

const appApi=new AppApi();

export default function FeaturesList() {

    let { section } = useParams();


    const featuresList=appApi.getFeaturesListInSection( section );

  return (
    <>
        {featuresList.map( 
            (featureObj) =>  

            <Card key={featureObj.name} style={{ width: '252px', float:'left', margin:'5px',  }}>
              <Card.Img variant="top" src={ appApi.getIcon(featureObj.section.path, featureObj.name)} />
              <Card.Footer  >
                <Card.Text>{featureObj.description}</Card.Text> 

                <div className="d-grid gap-2">
                <Button size="sm" variant="secondary" href={"/section/"+section+"/"+featureObj.name} to={"#section_"+section+'_'+featureObj.name}>Go</Button>
                </div>
              
              </Card.Footer>
                
            </Card>
      )}
    </>
  );
}
