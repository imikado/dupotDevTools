import { ListGroup } from "react-bootstrap";
import { Link } from "react-router-dom";

import FeatureApi from '../apis/FeatureApi';

const featuresApi=new FeatureApi();

export default function NavLeft(){

  const dirList= featuresApi.getSectionsList()

    return (
      
      <ListGroup  variant="flush">
        
        {dirList.map( (name)=><ListGroup.Item as={Link} key={name} to={"section/"+name} href={"#"+name} >{name}</ListGroup.Item>)}

      </ListGroup>
    )

}



