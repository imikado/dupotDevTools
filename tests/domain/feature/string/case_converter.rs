use dupot_dev_tools::domain::feature::string::case_converter::{
    Case, convert, convert_all, convert_lines, words,
};

#[test]
fn splits_words() {
    assert_eq!(words("HTTPServerError"), ["HTTP", "Server", "Error"]);
    assert_eq!(
        words("user_account-id name"),
        ["user", "account", "id", "name"]
    );
    assert_eq!(words("utf8Decode v2"), ["utf8", "Decode", "v2"]);
    assert_eq!(words("étéChaud"), ["été", "Chaud"]);
    assert!(words("  --  ").is_empty());
}

#[test]
fn converts_to_every_case() {
    let expected = [
        (Case::Camel, "httpServerError"),
        (Case::Pascal, "HttpServerError"),
        (Case::Snake, "http_server_error"),
        (Case::ScreamingSnake, "HTTP_SERVER_ERROR"),
        (Case::Kebab, "http-server-error"),
        (Case::ScreamingKebab, "HTTP-SERVER-ERROR"),
        (Case::Train, "Http-Server-Error"),
        (Case::Dot, "http.server.error"),
        (Case::Path, "http/server/error"),
        (Case::Title, "Http Server Error"),
        (Case::Sentence, "Http server error"),
        (Case::Lower, "http server error"),
        (Case::Upper, "HTTP SERVER ERROR"),
    ];
    for (case, out) in expected {
        assert_eq!(convert("HTTPServerError", case), out, "{case:?}");
    }
}

#[test]
fn converts_line_by_line() {
    assert_eq!(
        convert_lines("foo_bar\nBazQux", Case::Kebab),
        "foo-bar\nbaz-qux"
    );
    let all = convert_all("a b\n\nc");
    assert_eq!(all.split("\n\n").count(), 2);
    assert!(all.starts_with("camelCase             aB"), "{all}");
}
