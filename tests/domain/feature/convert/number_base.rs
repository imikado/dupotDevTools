use dupot_dev_tools::domain::feature::convert::number_base::{bit_width, describe, evaluate};

#[test]
fn reads_every_base() {
    for input in ["255", "0xFF", "0Xff", "0o377", "0b1111_1111"] {
        assert_eq!(evaluate(input).unwrap(), 255, "{input}");
    }
}

#[test]
fn follows_c_precedence() {
    assert_eq!(evaluate("1 | 2 & 3").unwrap(), 3);
    assert_eq!(evaluate("1 << 2 + 1").unwrap(), 8);
    assert_eq!(evaluate("(1 | 2) & 3").unwrap(), 3);
    assert_eq!(evaluate("0xFF ^ 0x0F").unwrap(), 0xF0);
    assert_eq!(evaluate("~0").unwrap(), -1);
    assert_eq!(evaluate("-2 * -3 % 4").unwrap(), 2);
    assert_eq!(evaluate("256 >> 4").unwrap(), 16);
}

#[test]
fn reports_errors() {
    assert!(evaluate("").is_err());
    assert!(evaluate("0xZZ").is_err());
    assert!(evaluate("1 +").is_err());
    assert!(evaluate("(1").is_err());
    assert!(evaluate("1 2").is_err());
    assert!(evaluate("1 / 0").is_err());
    assert!(evaluate("1 << 200").is_err());
    assert!(evaluate("170141183460469231731687303715884105727 + 1").is_err());
}

#[test]
fn widths_fit_the_value() {
    assert_eq!(bit_width(255), 8);
    assert_eq!(bit_width(256), 16);
    assert_eq!(bit_width(-128), 8);
    assert_eq!(bit_width(-129), 16);
    assert_eq!(bit_width(1 << 40), 64);
}

#[test]
fn describes_all_bases() {
    let out = describe("0x41").unwrap();
    assert!(out.contains("Decimal  65"), "{out}");
    assert!(out.contains("Hex      0x41"), "{out}");
    assert!(out.contains("Octal    0o101"), "{out}");
    assert!(out.contains("Binary   0100 0001"), "{out}");
    assert!(out.contains("'A'"), "{out}");
}

#[test]
fn describes_negatives_in_twos_complement() {
    let out = describe("-1").unwrap();
    assert!(out.contains("Hex (two's complement, 8-bit)"), "{out}");
    assert!(out.contains("0xFF"), "{out}");
    assert!(out.contains("Unsigned"), "{out}");
    assert!(describe("0xFF").unwrap().contains("Signed (8-bit)  -1"));
}
