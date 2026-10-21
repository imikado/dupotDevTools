use dupot_dev_tools::domain::error::{DomainError, DomainResult};

#[test]
fn new_keeps_the_message() {
    let e = DomainError::new("boom");
    assert_eq!(e.message(), "boom");
    assert_eq!(e.to_string(), "boom");
}

#[test]
fn library_errors_convert_with_question_mark() {
    fn parse() -> DomainResult<i32> {
        Ok("x".parse::<i32>()?)
    }
    assert_eq!(
        parse().unwrap_err().message(),
        "invalid digit found in string"
    );
}
