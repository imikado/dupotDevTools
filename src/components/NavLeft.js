import { Button, ButtonGroup } from "react-bootstrap";
import featuresApi from "../apis/featuresApi";


export default function NavLeft(){

  const dirList= featuresApi.getSectionsList()

    return (
        <ButtonGroup vertical>
         {dirList.map( (name)=><Button key={name} href={"/section/"+name} >{name}</Button>)}
     
      </ButtonGroup>
    )

}

