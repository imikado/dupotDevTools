import React from 'react';
import { Button, Card } from 'react-bootstrap';
import { Link, useParams } from 'react-router-dom';

import FeatureApi from '../apis/FeatureApi';

const featuresApi=new FeatureApi();

export default function FeaturesList() {

    let { section } = useParams();


    const featuresList=featuresApi.getFeaturesListInSection( section );

  return (
    <>
        {featuresList.map( 
            (featureObj) =>  

            <Card key={featureObj.name} style={{ width: '300px', float:'left', margin:'5px',  }}>
              <Card.Header>{featureObj.name}</Card.Header>
              <Card.Body  >
                <Card.Text>{featureObj.description}</Card.Text> 


                

                <Button variant="primary" href={"/section/"+section+"/"+featureObj.name} to={"#section_"+section+'_'+featureObj.name}>Go</Button>
              </Card.Body>
              <Card.Footer show={featureObj.comment}>{featureObj.comment}</Card.Footer> 
                
            </Card>
      )}
    </>
  );
}
