
import './App.css';
import 'bootstrap/dist/css/bootstrap.min.css';


import { Routes, Route } from "react-router-dom";


import { Col, Container, Row } from 'react-bootstrap';

import NavLeft from './components/NavLeft';
import Home from './components/Home';
import FeaturesList from './components/FeaturesList';

import Html2Text from './features/html/html2text/Html2Text'
import Page404 from './components/Page404';
import Hash from './features/string/hash/Hash';

export default function App() {
  return (
    <Container fluid>
      <Row>
        <Col xs={3}><NavLeft /></Col>
        <Col >
          <Routes>
            <Route path="/" element={<Home />} />

            <Route path="/section/html/html2text" element={<Html2Text />} />
            <Route path="/section/string/Hash" element={<Hash />} />

            
            <Route path="/section/:section" element={<FeaturesList />} />

            <Route path="*" element={<Page404 />} />
          </Routes>
        </Col>
      </Row>
    </Container>
    );
}
