import React from 'react';
import { Routes, Route } from "react-router-dom";


import Home from '../components/Home';
import FeaturesList from '../components/FeaturesList';
import Page404 from '../components/Page404';

import Html2Text from '../features/convert/htmlToText/Html2Text'
import Hash from '../features/string/hash/Hash';
import JsonFormat from '../features/formating/jsonFormat/JsonFormat';
import HtmlEncoding from '../features/encoding/htmlEncoding/HtmlEncoding';
import UrlEncoding from '../features/encoding/urlEncoding/UrlEncoding';
import YamlToJson from '../features/convert/yaml-Json/YamlToJson';
import XmlFormat from '../features/formating/xmlFormat/XmlFormat';
import SqlFormat from '../features/formating/sqlFormat/SqlFormat';
import GetTldr from '../features/helpful/tldr/GetTldr';

export default function Routing() {
  return (
    <Routes>
        <Route path="/" element={<Home />} />

        <Route path="/section/convert/htmlToText" element={<Html2Text />} />
        <Route path="/section/convert/yaml-Json" element={<YamlToJson />} />


        <Route path="/section/string/Hash" element={<Hash />} />

        <Route path="/section/formating/jsonFormat" element={<JsonFormat />} />
        <Route path="/section/formating/xmlFormat" element={<XmlFormat />} />
        <Route path="/section/formating/sqlFormat" element={<SqlFormat />} />
        
        <Route path="/section/encoding/htmlEncoding" element={<HtmlEncoding />} />
        <Route path="/section/encoding/urlEncoding" element={<UrlEncoding />} />

        <Route path="/section/helpful/tldr" element={<GetTldr />} />


        
        <Route path="/section/:section" element={<FeaturesList />} />

        <Route path="*" element={<Page404 />} />
    </Routes>
  );
}
