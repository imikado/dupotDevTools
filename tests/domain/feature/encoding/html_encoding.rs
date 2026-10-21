use dupot_dev_tools::domain::feature::encoding::html_encoding::{decode, encode};

#[test]
fn escapes_like_python_html_escape() {
    assert_eq!(
        encode("<a href=\"x\">'&'</a>"),
        "&lt;a href=&quot;x&quot;&gt;&#x27;&amp;&#x27;&lt;/a&gt;"
    );
}

#[test]
fn decodes_named_and_numeric_entities() {
    assert_eq!(decode("&lt;&eacute;&#233;&#x27;&gt;"), "<éé'>");
}
