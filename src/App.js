
import 'bootstrap/dist/css/bootstrap.min.css';

import { Col, Container, Row } from 'react-bootstrap';

import NavLeft from './components/NavLeft';
import Routing from './conf/Routing';


export default function App() {
  return (
    <Container fluid>
      <Row>
        <Col xs={3}><NavLeft /></Col>
        <Col >
          <Routing/>
        </Col>
      </Row>
    </Container>
    );
}
