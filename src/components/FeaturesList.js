import React from 'react';
import { Button, Card } from 'react-bootstrap';
import { Link, useParams } from 'react-router-dom';
import featuresApi from '../apis/featuresApi';



export default function FeaturesList() {

    let { section } = useParams();


    const featuresList=featuresApi.getFeaturesListInSection( section );

  return (
    <>
        {featuresList.map( 
            (featureObj) =>  

            <Card key={featureObj.name} style={{ width: '18rem' }}>
              <Card.Header>{featureObj.name}</Card.Header>
              <Card.Body  >
                <Card.Text>{featureObj.description}</Card.Text> 
                <Button variant="primary" href={"/section/"+section+"/"+featureObj.name} to={"#section_"+section+'_'+featureObj.name}>Go</Button>
              </Card.Body>
            </Card>
      )}
    </>
  );
}
