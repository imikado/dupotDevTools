import { ListGroup } from "react-bootstrap";
import { Link } from "react-router-dom";

import AppApi from '../apis/AppApi';

const appApi=new AppApi();

export default function NavLeft(){

  const dirList= appApi.getSectionsList()

    return (
      
      <ListGroup  variant="flush">
        
        {dirList.map( (name)=><ListGroup.Item as={Link} key={name} to={"section/"+name} href={"#"+name} >{name}</ListGroup.Item>)}

      </ListGroup>
    )

}



