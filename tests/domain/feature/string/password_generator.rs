use dupot_dev_tools::domain::feature::string::password_generator::{
    PasswordOptions, crack_time, generate, generate_with, inspect, secure_below, strength,
};

#[test]
fn generates_the_requested_shape() {
    let options = PasswordOptions {
        length: 16,
        ..Default::default()
    };
    let out = generate(options, 5).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 5);
    for line in lines {
        assert_eq!(line.chars().count(), 16);
        assert!(line.chars().any(|c| c.is_ascii_lowercase()));
        assert!(line.chars().any(|c| c.is_ascii_uppercase()));
        assert!(line.chars().any(|c| c.is_ascii_digit()));
        assert!(line.chars().any(|c| c.is_ascii_punctuation()));
    }
}

#[test]
fn respects_classes_and_ambiguous_exclusion() {
    let options = PasswordOptions {
        length: 200,
        lowercase: false,
        uppercase: true,
        digits: true,
        symbols: false,
        exclude_ambiguous: true,
    };
    let out = generate(options, 1).unwrap();
    assert!(
        out.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    );
    assert!(!out.contains(['I', 'O', '0', '1']));
    assert_eq!(options.pool_size(), 24 + 8);
}

#[test]
fn redraws_until_every_class_is_present() {
    // A source that first yields "aaaa", then "aA0b" (pool: a-z A-Z 0-9).
    let mut calls = 0;
    let mut below = |_: usize| {
        calls += 1;
        if calls <= 4 {
            0
        } else {
            [0, 26, 52, 1][(calls - 1) % 4]
        }
    };
    let options = PasswordOptions {
        length: 4,
        symbols: false,
        ..Default::default()
    };
    let out = generate_with(options, 1, &mut below).unwrap();
    assert_eq!(out, "aA0b");
    assert!(calls > 4);
}

#[test]
fn rejects_impossible_options() {
    let none = PasswordOptions {
        lowercase: false,
        uppercase: false,
        digits: false,
        symbols: false,
        ..Default::default()
    };
    assert!(generate(none, 1).is_err());
    let short = PasswordOptions {
        length: 3,
        ..Default::default()
    };
    assert!(generate(short, 1).is_err());
}

#[test]
fn secure_below_stays_in_range() {
    assert!((0..1000).all(|_| secure_below(7) < 7));
}

#[test]
fn estimates_entropy() {
    let options = PasswordOptions::default();
    assert!((options.entropy_bits() - 20.0 * 94f64.log2()).abs() < 1e-9);
    assert_eq!(strength(20.0), "Very weak");
    assert_eq!(strength(80.0), "Strong");
    assert_eq!(crack_time(1.0), "instantly");
    assert_eq!(crack_time(128.0), "longer than the age of the universe");
    assert_eq!(crack_time(50.0), "15 hours");
}

#[test]
fn inspects_passwords() {
    let weak = inspect("Password123");
    assert!(weak.contains("common password \"password\""), "{weak}");
    assert!(weak.contains("Very weak"), "{weak}");
    let strong = inspect("v9#Lq2!xR7@pZ4&w");
    assert!(strong.contains("16 characters"), "{strong}");
    assert!(strong.contains("95 possible characters"), "{strong}");
    assert!(strong.contains("Strong"), "{strong}");
    assert!(inspect("aaaaaaaaaaaaaaaa").contains("repeated"));
}
