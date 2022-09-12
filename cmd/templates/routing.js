import React from 'react';
import { Routes, Route } from "react-router-dom";


import Home from '../components/Home';
import FeaturesList from '../components/FeaturesList';
import Page404 from '../components/Page404';

//#IMPORT_FEATURES_LIST#
//import #FeatureClass# from '../features/#section#/#feature#/#FeatureClass#';

//ROUTES LOOP
//<Route path="/section/#section#/#feature#" element={<#FeatureClass# />} />

export default function Routing() {
  return (
    <Routes>
        <Route path="/" element={<Home />} />
        
<ROUTES_LOOP></ROUTES_LOOP>
        
        <Route path="/section/:section" element={<FeaturesList />} />

        <Route path="*" element={<Page404 />} />
    </Routes>
  );
}
