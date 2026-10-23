use dupot_dev_tools::domain::feature::convert::color::{contrast_ratio, describe, parse};

#[test]
fn parses_every_notation_to_the_same_color() {
    for input in [
        "#3584e4",
        "3584E4",
        "rgb(53 132 228)",
        "rgb(53, 132, 228)",
        "rgba(53,132,228,1)",
        "rgb(20.8% 51.8% 89.4%)",
    ] {
        assert_eq!(parse(input).unwrap().hex(), "#3584E4", "{input}");
    }
    assert_eq!(parse("#38e").unwrap().hex(), "#3388EE");
    assert_eq!(parse("hsl(0 100% 50%)").unwrap().hex(), "#FF0000");
    assert_eq!(parse("hsl(120deg, 100%, 25%)").unwrap().hex(), "#008000");
    assert_eq!(parse("hsla(240 100 50 / 0.5)").unwrap().hex(), "#0000FF80");
}

#[test]
fn reads_alpha() {
    assert_eq!(parse("#3584e480").unwrap().hex(), "#3584E480");
    assert_eq!(parse("rgb(53 132 228 / 50%)").unwrap().alpha, 0.5);
}

#[test]
fn oklch_round_trips() {
    let color = parse("#3584e4").unwrap();
    let (l, c, h) = color.oklch();
    let back = parse(&format!("oklch({l} {c} {h})")).unwrap();
    assert_eq!(back.hex(), "#3584E4");
    assert!(!back.clipped);
    assert!(parse("oklch(70% 0.4 150)").unwrap().clipped);
}

#[test]
fn computes_wcag_contrast() {
    let white = parse("#fff").unwrap();
    let black = parse("#000").unwrap();
    assert!((contrast_ratio(white, black) - 21.0).abs() < 1e-9);
    let ratio = contrast_ratio(parse("#767676").unwrap(), white);
    assert!((ratio - 4.54).abs() < 0.01, "{ratio}");
}

#[test]
fn describes_formats_and_contrast() {
    let out = describe("#ffffff", Some("#767676")).unwrap();
    assert!(out.contains("rgb(255 255 255)"), "{out}");
    assert!(out.contains("hsl(0 0% 100%)"), "{out}");
    assert!(out.contains("oklch(100% 0 0)"), "{out}");
    assert!(out.contains("vs black    21:1  AA ✓  AAA ✓"), "{out}");
    assert!(out.contains("vs #767676  4.54:1  AA ✓  AAA ✗"), "{out}");
}

#[test]
fn rejects_unknown_colors() {
    assert!(parse("").is_err());
    assert!(parse("#12").is_err());
    assert!(parse("rgb(1 2)").is_err());
    assert!(
        parse("blue-ish")
            .unwrap_err()
            .message()
            .contains("Accepted")
    );
}
