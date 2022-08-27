
import './App.css';
import 'bootstrap/dist/css/bootstrap.min.css';


import { Routes, Route } from "react-router-dom";


import { Col, Container, Row } from 'react-bootstrap';

import NavLeft from './components/NavLeft';
import Home from './components/Home';
import FeaturesList from './components/FeaturesList';
import Page404 from './components/Page404';

import Html2Text from './features/convert/htmlToText/Html2Text'
import Hash from './features/string/hash/Hash';
import JsonFormat from './features/formating/jsonFormat/JsonFormat';
import HtmlEncoding from './features/encoding/htmlEncoding/HtmlEncoding';
import UrlEncoding from './features/encoding/urlEncoding/UrlEncoding';
import YamlToJson from './features/convert/yaml-Json/YamlToJson';
import XmlFormat from './features/formating/xmlFormat/XmlFormat';

export default function App() {
  return (
    <Container fluid>
      <Row>
        <Col xs={3}><NavLeft /></Col>
        <Col >
          <Routes>
            <Route path="/" element={<Home />} />

            <Route path="/section/convert/htmlToText" element={<Html2Text />} />
            <Route path="/section/convert/yaml-Json" element={<YamlToJson />} />


            <Route path="/section/string/Hash" element={<Hash />} />

            <Route path="/section/formating/jsonFormat" element={<JsonFormat />} />
            <Route path="/section/formating/xmlFormat" element={<XmlFormat />} />
            
            <Route path="/section/encoding/htmlEncoding" element={<HtmlEncoding />} />
            <Route path="/section/encoding/urlEncoding" element={<UrlEncoding />} />

            
            <Route path="/section/:section" element={<FeaturesList />} />

            <Route path="*" element={<Page404 />} />
          </Routes>
        </Col>
      </Row>
    </Container>
    );
}
