use chrono::{DateTime, Utc};
use dupot_dev_tools::domain::feature::encoding::jwt_sign::{
    JwtAlgorithm, SAMPLE_PAYLOAD, sign, sign_or_verify, verify,
};

const RSA_PRIVATE: &str = include_str!("../../../fixtures/jwt/rsa.pem");
const RSA_PUBLIC: &str = include_str!("../../../fixtures/jwt/rsa.pub");
const EC_PRIVATE: &str = include_str!("../../../fixtures/jwt/ec.pem");
const EC_PUBLIC: &str = include_str!("../../../fixtures/jwt/ec.pub");
const ED_PRIVATE: &str = include_str!("../../../fixtures/jwt/ed.pem");
const ED_PUBLIC: &str = include_str!("../../../fixtures/jwt/ed.pub");

/// The jwt.io example token.
const JWT_IO: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
    eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.\
    SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

fn now() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).unwrap()
}

#[test]
fn hs256_matches_the_reference_token() {
    let token = sign(SAMPLE_PAYLOAD, JwtAlgorithm::Hs256, "your-256-bit-secret").unwrap();
    assert_eq!(token, JWT_IO);
    let report = verify(JWT_IO, "your-256-bit-secret", now()).unwrap();
    assert!(report.contains("✓ valid (HS256)"), "{report}");
    assert!(report.contains("\"name\": \"John Doe\""), "{report}");
    assert!(
        verify(JWT_IO, "wrong", now())
            .unwrap_err()
            .message()
            .contains("Invalid signature")
    );
}

#[test]
fn asymmetric_round_trips() {
    let cases = [
        (JwtAlgorithm::Rs256, RSA_PRIVATE, RSA_PUBLIC),
        (JwtAlgorithm::Rs512, RSA_PRIVATE, RSA_PUBLIC),
        (JwtAlgorithm::Ps256, RSA_PRIVATE, RSA_PUBLIC),
        (JwtAlgorithm::Es256, EC_PRIVATE, EC_PUBLIC),
        (JwtAlgorithm::EdDsa, ED_PRIVATE, ED_PUBLIC),
    ];
    for (alg, private, public) in cases {
        let token = sign(r#"{"sub":"42"}"#, alg, private).unwrap();
        for key in [public, private] {
            let report = verify(&token, key, now()).unwrap_or_else(|e| panic!("{alg:?}: {e}"));
            assert!(
                report.contains(&format!("✓ valid ({})", alg.label())),
                "{report}"
            );
        }
    }
}

#[test]
fn rejects_tampered_tokens_and_wrong_keys() {
    let token = sign(r#"{"admin":false}"#, JwtAlgorithm::Rs256, RSA_PRIVATE).unwrap();
    let parts: Vec<&str> = token.split('.').collect();
    let forged = format!("{}.eyJhZG1pbiI6dHJ1ZX0.{}", parts[0], parts[2]);
    assert!(verify(&forged, RSA_PUBLIC, now()).is_err());
    assert!(verify(&token, EC_PUBLIC, now()).is_err());
    assert!(sign("{}", JwtAlgorithm::Es256, RSA_PRIVATE).is_err());
}

#[test]
fn reports_time_claims() {
    let token = sign(r#"{"exp":1600000000}"#, JwtAlgorithm::Hs256, "s").unwrap();
    let report = verify(&token, "s", now()).unwrap();
    assert!(
        report.contains("Expires    2020-09-13 12:26:40 UTC (3 years ago)"),
        "{report}"
    );
    assert!(report.contains("⚠ expired"), "{report}");
}

#[test]
fn validates_inputs() {
    assert!(sign("[1]", JwtAlgorithm::Hs256, "s").is_err());
    assert!(sign("{}", JwtAlgorithm::Hs256, "").is_err());
    assert!(sign("{}", JwtAlgorithm::Rs256, "not a pem").is_err());
    assert!(verify("a.b", "s", now()).is_err());
    let none = "eyJhbGciOiJub25lIn0.e30.";
    assert!(
        verify(none, "s", now())
            .unwrap_err()
            .message()
            .contains("Unsupported")
    );
}

#[test]
fn signs_json_and_verifies_tokens() {
    let token = sign_or_verify(
        SAMPLE_PAYLOAD,
        JwtAlgorithm::Hs256,
        "your-256-bit-secret",
        now(),
    );
    assert_eq!(token.unwrap(), JWT_IO);
    let report = sign_or_verify(JWT_IO, JwtAlgorithm::Rs256, "your-256-bit-secret", now());
    assert!(report.unwrap().contains("✓ valid (HS256)"));
}
