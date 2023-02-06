import {
  Box,
  Button,
  Card,
  CardActionArea,
  CardContent,
  CardMedia,
  Grid,
  Typography,
} from "@mui/material";
import { Link as RouterLink, useParams } from "react-router-dom";

import AppApi from "../apis/AppApi";
const appApi = new AppApi();

export default function Home() {
  //const sectionList= appApi.getSectionsList()

  const featuresList = appApi.getFeaturesList();

  return (
    <Grid container columnSpacing={{ xs: 1, sm: 2, md: 3 }} rowSpacing={3}>
      {featuresList.map((featureObj) => (
        <Grid key={featureObj.name} item xs={4}>
          <Card>
            <CardActionArea
              component={RouterLink}
              href={
                "#section_" + featureObj.section.path + "_" + featureObj.name
              }
              to={"/section/" + featureObj.section.path + "/" + featureObj.name}
            >
              <CardMedia
                component="img"
                height="250"
                image={appApi.getIconForSectionAndFeature(
                  featureObj.section.path,
                  featureObj.name
                )}
                alt="green iguana"
              />
              <CardContent>
                <Typography gutterBottom variant="h5" component="div">
                  {featureObj.name}
                </Typography>
                <Typography variant="body2" color="text.secondary"></Typography>
              </CardContent>
            </CardActionArea>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
}
