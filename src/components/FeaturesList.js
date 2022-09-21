import { Box, Button, Card, CardActionArea, CardContent, CardMedia, Grid, Typography } from '@mui/material';
import React from 'react';
import { Link, useParams } from 'react-router-dom';

import AppApi from '../apis/AppApi';

const appApi=new AppApi();

export default function FeaturesList() {

    let { section } = useParams();


    const featuresList=appApi.getFeaturesListInSection( section );

  return (
    <Grid container columnSpacing={{ xs: 1, sm: 2, md: 3 }} rowSpacing={3}>
        {featuresList.map( 
            (featureObj) =>  

            <Grid key={featureObj.name}  item xs={4}>
            <Card >
              <CardActionArea
              href={"/section/"+section+"/"+featureObj.name} to={"#section_"+section+'_'+featureObj.name}
              >
                <CardMedia
                  component="img"
                  height="250"
                  image={ appApi.getIcon(featureObj.section.path, featureObj.name)}
                  alt="green iguana"


                />
                <CardContent>
                  <Typography gutterBottom variant="h5" component="div">
                    {featureObj.name}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                  {featureObj.description}
                  </Typography>
                </CardContent>
              </CardActionArea>
            </Card>
            </Grid>


      )}
    </Grid>
  );
}
