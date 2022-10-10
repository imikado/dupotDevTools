import { ButtonGroup,Button, Box } from "@mui/material";
import { useState } from "react";
import { Link as RouterLink} from "react-router-dom";

import AppApi from '../apis/AppApi';

const appApi=new AppApi();

export default function NavLeft(){

  const [linkSelected, setLinkSelected] = useState("");

  const sectionList= appApi.getSectionsList()

    return (
    

      <ButtonGroup
      orientation="vertical"
      aria-label="vertical outlined button group"
      variant="outlined"
    >
             {sectionList.map( 
                (name)=>
                  <Button key={name} 
                  
                  variant={ linkSelected===name?"contained":"outlined"}

                  onClick={()=>setLinkSelected(name)}
                  
                  component={RouterLink} to={"section/"+name} href={"#section/"+name} >{name}</Button>
              )}

    </ButtonGroup>
    
    )

};



