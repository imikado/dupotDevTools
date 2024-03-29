import React from 'react';
import { Routes, Route } from "react-router-dom";


import Home from '../components/Home';
import FeaturesList from '../components/FeaturesList';
import Page404 from '../components/Page404';

import AddFeatureFeature from '../features/app/addFeature/AddFeatureFeature'; 
import Html2TextFeature from '../features/convert/htmlToText/Html2TextFeature'; 
import HtmltopdfFeature from '../features/convert/htmltopdf/HtmltopdfFeature'; 
import YamlToJsonFeature from '../features/convert/yaml-Json/YamlToJsonFeature'; 
import NfcToolFeature from '../features/devices/nfcTool/NfcToolFeature'; 
import Base64encodingFeature from '../features/encoding/base64encoding/Base64encodingFeature'; 
import HtmlEncodingFeature from '../features/encoding/htmlEncoding/HtmlEncodingFeature'; 
import JwtdecodeFeature from '../features/encoding/jwtDecode/JwtdecodeFeature'; 
import UrlEncodingFeature from '../features/encoding/urlEncoding/UrlEncodingFeature'; 
import GraphqlFormatFeature from '../features/formating/graphqlFormat/GraphqlFormatFeature'; 
import JsonFormatFeature from '../features/formating/jsonFormat/JsonFormatFeature'; 
import SqlFormatFeature from '../features/formating/sqlFormat/SqlFormatFeature'; 
import XmlFormatFeature from '../features/formating/xmlFormat/XmlFormatFeature'; 
import GraphvizFeature from '../features/graphics/graphviz/GraphvizFeature'; 
import GetTldrFeature from '../features/helpful/tldr/GetTldrFeature'; 
import XsvparsingcsvFeature from '../features/parsing/xsv-parsing-csv/XsvparsingcsvFeature'; 
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
<Route path="/section/devices/nfcTool/" element={<NfcToolFeature />} />  
<Route path="/section/encoding/base64encoding/" element={<Base64encodingFeature />} />  
<Route path="/section/encoding/htmlEncoding/" element={<HtmlEncodingFeature />} />  
<Route path="/section/encoding/jwtDecode/" element={<JwtdecodeFeature />} />  
<Route path="/section/encoding/urlEncoding/" element={<UrlEncodingFeature />} />  
<Route path="/section/formating/graphqlFormat/" element={<GraphqlFormatFeature />} />  
<Route path="/section/formating/jsonFormat/" element={<JsonFormatFeature />} />  
<Route path="/section/formating/sqlFormat/" element={<SqlFormatFeature />} />  
<Route path="/section/formating/xmlFormat/" element={<XmlFormatFeature />} />  
<Route path="/section/graphics/graphviz/" element={<GraphvizFeature />} />  
<Route path="/section/helpful/tldr/" element={<GetTldrFeature />} />  
<Route path="/section/parsing/xsv-parsing-csv/" element={<XsvparsingcsvFeature />} />  
<Route path="/section/string/hash/" element={<HashFeature />} />  
        
        <Route path="/section/:section" element={<FeaturesList />} />

        <Route path="*" element={<Home />} />

    </Routes>
  );
}
