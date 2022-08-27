import React from 'react';
import { Button } from 'react-bootstrap';
import { useParams } from 'react-router-dom';
import featuresApi from '../apis/featuresApi';



export default function FeaturesList() {

    let { section } = useParams();


    const featuresList=featuresApi.getFeaturesListInSection( section );

  return (
    <div>
        {featuresList.map( 
            (value) => 
                <Button key={value} href={"/section/"+section+"/"+value}>{value}</Button>)}
      
    </div>
  );
}
