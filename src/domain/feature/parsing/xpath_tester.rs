use sxd_document::dom::{ChildOfElement, Element};
use sxd_xpath::nodeset::Node;
use sxd_xpath::{Context, Factory, Value};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::shared::format::number;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "xpath_tester",
    section: "parsing",
    title: "XPath Tester",
    description: "Run XPath 1.0 expressions on XML",
};

pub const SAMPLE: &str = r#"<?xml version="1.0"?>
<library>
    <book id="1" lang="en">
        <title>Dune</title>
        <author>Frank Herbert</author>
        <price>9.99</price>
    </book>
    <book id="2" lang="fr">
        <title>Le Petit Prince</title>
        <author>Antoine de Saint-Exupéry</author>
        <price>7.50</price>
    </book>
</library>"#;

fn name(prefix: Option<&str>, local: &str) -> String {
    match prefix {
        Some(p) => format!("{p}:{local}"),
        None => local.to_string(),
    }
}

fn write_element(element: Element, out: &mut String) {
    let tag = name(element.preferred_prefix(), element.name().local_part());
    out.push('<');
    out.push_str(&tag);
    for attr in element.attributes() {
        let attr_name = name(attr.preferred_prefix(), attr.name().local_part());
        out.push_str(&format!(
            " {attr_name}=\"{}\"",
            html_escape::encode_double_quoted_attribute(attr.value())
        ));
    }
    let children = element.children();
    if children.is_empty() {
        out.push_str("/>");
        return;
    }
    out.push('>');
    for child in children {
        match child {
            ChildOfElement::Element(e) => write_element(e, out),
            ChildOfElement::Text(t) => out.push_str(&html_escape::encode_text(t.text())),
            ChildOfElement::Comment(c) => out.push_str(&format!("<!--{}-->", c.text())),
            ChildOfElement::ProcessingInstruction(pi) => {
                out.push_str(&format!("<?{} {}?>", pi.target(), pi.value().unwrap_or("")))
            }
        }
    }
    out.push_str(&format!("</{tag}>"));
}

fn node_to_string(node: Node) -> String {
    match node {
        Node::Element(e) => {
            let mut out = String::new();
            write_element(e, &mut out);
            out
        }
        Node::Attribute(a) => format!(
            "{}=\"{}\"",
            name(a.preferred_prefix(), a.name().local_part()),
            html_escape::encode_double_quoted_attribute(a.value())
        ),
        Node::Comment(c) => format!("<!--{}-->", c.text()),
        other => other.string_value(),
    }
}

/// The expression's value: matched nodes serialized as XML, one per line, or
/// the boolean, number or string result.
pub fn evaluate(expr: &str, xml: &str) -> DomainResult<String> {
    if expr.trim().is_empty() {
        return Err(DomainError::new("Enter an XPath expression."));
    }
    let package = sxd_document::parser::parse(xml)
        .map_err(|e| DomainError::new(format!("Invalid XML: {e}")))?;
    let document = package.as_document();
    let xpath = Factory::new()
        .build(expr.trim())
        .map_err(|e| DomainError::new(format!("Invalid XPath: {e}")))?
        .ok_or_else(|| DomainError::new("Enter an XPath expression."))?;
    let value = xpath
        .evaluate(&Context::new(), document.root())
        .map_err(|e| DomainError::new(format!("XPath error: {e}")))?;
    Ok(match value {
        Value::Nodeset(nodes) => {
            let nodes = nodes.document_order();
            let plural = if nodes.len() == 1 { "" } else { "es" };
            let mut out = format!("<!-- {} match{plural} -->", nodes.len());
            for node in nodes {
                out.push('\n');
                out.push_str(&node_to_string(node));
            }
            out
        }
        Value::Boolean(b) => b.to_string(),
        Value::Number(n) => number(n, 10),
        Value::String(s) => s,
    })
}
