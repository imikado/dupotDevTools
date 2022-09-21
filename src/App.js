
import {  Grid } from '@mui/material';

import NavLeft from './components/NavLeft';
import Routing from './conf/Routing';


export default function App() {
  return (
    <Grid container >

      <Grid item  xs ><NavLeft /></Grid>
      <Grid item xs={10}  >
          <Routing/>
        </Grid>
      </Grid>
    );
}
