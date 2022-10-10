import React from 'react';
import { Routes, Route } from "react-router-dom";


import Home from '../components/Home';
import FeaturesList from '../components/FeaturesList';
import Page404 from '../components/Page404';

import AddFeatureFeature from '../features/app/addFeature/AddFeatureFeature'; 
import Html2TextFeature from '../features/convert/htmlToText/Html2TextFeature'; 
import HtmltopdfFeature from '../features/convert/htmltopdf/HtmltopdfFeature'; 
import YamlToJsonFeature from '../features/convert/yaml-Json/YamlToJsonFeature'; 
import HtmlEncodingFeature from '../features/encoding/htmlEncoding/HtmlEncodingFeature'; 
import UrlEncodingFeature from '../features/encoding/urlEncoding/UrlEncodingFeature'; 
import JsonFormatFeature from '../features/formating/jsonFormat/JsonFormatFeature'; 
import SqlFormatFeature from '../features/formating/sqlFormat/SqlFormatFeature'; 
import XmlFormatFeature from '../features/formating/xmlFormat/XmlFormatFeature'; 
import GetTldrFeature from '../features/helpful/tldr/GetTldrFeature'; 
import HashFeature from '../features/string/hash/HashFeature'; 
//import #FeatureClass# from '../features/#section#/#feature#/#FeatureClass#';

//ROUTES LOOP
//<Route path="/section/#section#/#feature#" element={<#FeatureClass# />} />

export default function Routing() {
  return (
    <Routes>
        
<Route path="/section/app/addFeature/" element={<AddFeatureFeature />} />  
<Route path="/section/convert/htmlToText/" element={<Html2TextFeature />} />  
<Route path="/section/convert/htmltopdf/" element={<HtmltopdfFeature />} />  
<Route path="/section/convert/yaml-Json/" element={<YamlToJsonFeature />} />  
<Route path="/section/encoding/htmlEncoding/" element={<HtmlEncodingFeature />} />  
<Route path="/section/encoding/urlEncoding/" element={<UrlEncodingFeature />} />  
<Route path="/section/formating/jsonFormat/" element={<JsonFormatFeature />} />  
<Route path="/section/formating/sqlFormat/" element={<SqlFormatFeature />} />  
<Route path="/section/formating/xmlFormat/" element={<XmlFormatFeature />} />  
<Route path="/section/helpful/tldr/" element={<GetTldrFeature />} />  
<Route path="/section/string/hash/" element={<HashFeature />} />  
        
        <Route path="/section/:section" element={<FeaturesList />} />

        <Route path="*" element={<Home />} />

    </Routes>
  );
}
