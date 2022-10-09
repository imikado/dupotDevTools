import { ButtonGroup,Button, Box } from "@mui/material";
import { Link as RouterLink} from "react-router-dom";

import AppApi from '../apis/AppApi';

const appApi=new AppApi();

export default function NavLeft(){

  const sectionList= appApi.getSectionsList()

    return (
    

      <ButtonGroup
      orientation="vertical"
      aria-label="vertical outlined button group"
      variant="contained"
    >
             {sectionList.map( 
                (name)=>
                  <Button key={name} component={RouterLink} to={"section/"+name} href={"#section/"+name} >{name}</Button>
              )}

    </ButtonGroup>
    
    )

};



