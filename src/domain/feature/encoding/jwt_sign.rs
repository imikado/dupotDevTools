use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use ring::hmac;
use ring::rand::SystemRandom;
use ring::signature::{self, EcdsaKeyPair, Ed25519KeyPair, KeyPair, RsaKeyPair};
use serde_json::{Map, Value, json};

use crate::domain::entity::feature_entity::FeatureEntity;
use crate::domain::error::{DomainError, DomainResult};
use crate::domain::feature::convert::timestamp::relative;
use crate::domain::shared::format::rows;
use crate::domain::shared::json::to_pretty_json;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "jwt_sign",
    section: "encoding",
    title: "JWT Encoder / Verifier",
    description: "Sign JWT tokens (HS, RS, PS, ES, EdDSA) and verify their signature",
};

pub const SAMPLE_PAYLOAD: &str =
    "{\n    \"sub\": \"1234567890\",\n    \"name\": \"John Doe\",\n    \"iat\": 1516239022\n}";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JwtAlgorithm {
    Hs256,
    Hs384,
    Hs512,
    Rs256,
    Rs384,
    Rs512,
    Ps256,
    Ps384,
    Ps512,
    Es256,
    Es384,
    EdDsa,
}

impl JwtAlgorithm {
    pub const ALL: [JwtAlgorithm; 12] = [
        JwtAlgorithm::Hs256,
        JwtAlgorithm::Hs384,
        JwtAlgorithm::Hs512,
        JwtAlgorithm::Rs256,
        JwtAlgorithm::Rs384,
        JwtAlgorithm::Rs512,
        JwtAlgorithm::Ps256,
        JwtAlgorithm::Ps384,
        JwtAlgorithm::Ps512,
        JwtAlgorithm::Es256,
        JwtAlgorithm::Es384,
        JwtAlgorithm::EdDsa,
    ];

    pub fn label(self) -> &'static str {
        match self {
            JwtAlgorithm::Hs256 => "HS256",
            JwtAlgorithm::Hs384 => "HS384",
            JwtAlgorithm::Hs512 => "HS512",
            JwtAlgorithm::Rs256 => "RS256",
            JwtAlgorithm::Rs384 => "RS384",
            JwtAlgorithm::Rs512 => "RS512",
            JwtAlgorithm::Ps256 => "PS256",
            JwtAlgorithm::Ps384 => "PS384",
            JwtAlgorithm::Ps512 => "PS512",
            JwtAlgorithm::Es256 => "ES256",
            JwtAlgorithm::Es384 => "ES384",
            JwtAlgorithm::EdDsa => "EdDSA",
        }
    }

    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|a| a.label() == label)
    }

    /// Signed with a shared secret rather than a key pair.
    pub fn is_hmac(self) -> bool {
        matches!(
            self,
            JwtAlgorithm::Hs256 | JwtAlgorithm::Hs384 | JwtAlgorithm::Hs512
        )
    }

    fn hmac(self) -> hmac::Algorithm {
        match self {
            JwtAlgorithm::Hs384 => hmac::HMAC_SHA384,
            JwtAlgorithm::Hs512 => hmac::HMAC_SHA512,
            _ => hmac::HMAC_SHA256,
        }
    }

    fn rsa_padding(self) -> &'static dyn signature::RsaEncoding {
        match self {
            JwtAlgorithm::Rs384 => &signature::RSA_PKCS1_SHA384,
            JwtAlgorithm::Rs512 => &signature::RSA_PKCS1_SHA512,
            JwtAlgorithm::Ps256 => &signature::RSA_PSS_SHA256,
            JwtAlgorithm::Ps384 => &signature::RSA_PSS_SHA384,
            JwtAlgorithm::Ps512 => &signature::RSA_PSS_SHA512,
            _ => &signature::RSA_PKCS1_SHA256,
        }
    }

    fn ecdsa_signing(self) -> &'static signature::EcdsaSigningAlgorithm {
        match self {
            JwtAlgorithm::Es384 => &signature::ECDSA_P384_SHA384_FIXED_SIGNING,
            _ => &signature::ECDSA_P256_SHA256_FIXED_SIGNING,
        }
    }

    fn verification(self) -> &'static dyn signature::VerificationAlgorithm {
        match self {
            JwtAlgorithm::Rs256 => &signature::RSA_PKCS1_2048_8192_SHA256,
            JwtAlgorithm::Rs384 => &signature::RSA_PKCS1_2048_8192_SHA384,
            JwtAlgorithm::Rs512 => &signature::RSA_PKCS1_2048_8192_SHA512,
            JwtAlgorithm::Ps256 => &signature::RSA_PSS_2048_8192_SHA256,
            JwtAlgorithm::Ps384 => &signature::RSA_PSS_2048_8192_SHA384,
            JwtAlgorithm::Ps512 => &signature::RSA_PSS_2048_8192_SHA512,
            JwtAlgorithm::Es256 => &signature::ECDSA_P256_SHA256_FIXED,
            JwtAlgorithm::Es384 => &signature::ECDSA_P384_SHA384_FIXED,
            _ => &signature::ED25519,
        }
    }
}

/// A PEM block: its label (`PRIVATE KEY`, `PUBLIC KEY`…) and DER content.
struct Pem {
    label: String,
    der: Vec<u8>,
}

fn parse_pem(text: &str) -> DomainResult<Pem> {
    let no_pem = || DomainError::new("Expected a PEM key (-----BEGIN … KEY-----).");
    let (_, after_begin) = text.split_once("-----BEGIN ").ok_or_else(no_pem)?;
    let (label, body) = after_begin.split_once("-----").ok_or_else(no_pem)?;
    let (body, _) = body.split_once("-----END").ok_or_else(no_pem)?;
    let base64: String = body.split_whitespace().collect();
    Ok(Pem {
        label: label.trim().to_string(),
        der: STANDARD.decode(base64)?,
    })
}

/// One DER element: (tag, content, rest).
fn der_element(data: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let (&tag, data) = data.split_first()?;
    let (&first, mut data) = data.split_first()?;
    let len = if first < 0x80 {
        first as usize
    } else {
        let count = (first & 0x7f) as usize;
        if count == 0 || count > 4 || data.len() < count {
            return None;
        }
        let len = data[..count]
            .iter()
            .fold(0usize, |acc, b| acc << 8 | *b as usize);
        data = &data[count..];
        len
    };
    (data.len() >= len).then(|| (tag, &data[..len], &data[len..]))
}

/// The raw key inside a SubjectPublicKeyInfo: PKCS#1 for RSA, the point for
/// EC, the 32 bytes for Ed25519.
fn spki_key(der: &[u8]) -> Option<&[u8]> {
    let (0x30, spki, _) = der_element(der)? else {
        return None;
    };
    let (0x30, _, rest) = der_element(spki)? else {
        return None;
    };
    let (0x03, bits, _) = der_element(rest)? else {
        return None;
    };
    bits.strip_prefix(&[0])
}

fn rsa_key_pair(pem: &Pem) -> DomainResult<RsaKeyPair> {
    let pair = match pem.label.as_str() {
        "PRIVATE KEY" => RsaKeyPair::from_pkcs8(&pem.der),
        "RSA PRIVATE KEY" => RsaKeyPair::from_der(&pem.der),
        _ => return Err(DomainError::new("Signing needs an RSA private key (PEM).")),
    };
    pair.map_err(|e| DomainError::new(format!("RSA key rejected: {e}")))
}

fn pkcs8<'a>(pem: &'a Pem, kind: &str) -> DomainResult<&'a [u8]> {
    match pem.label.as_str() {
        "PRIVATE KEY" => Ok(&pem.der),
        "EC PRIVATE KEY" => Err(DomainError::new(
            "SEC1 EC keys are not supported, convert to PKCS#8 first:\n\
             openssl pkcs8 -topk8 -nocrypt -in key.pem",
        )),
        _ => Err(DomainError::new(format!(
            "Signing needs an {kind} private key in PKCS#8 PEM (-----BEGIN PRIVATE KEY-----)."
        ))),
    }
}

fn ecdsa_key_pair(alg: JwtAlgorithm, pem: &Pem) -> DomainResult<EcdsaKeyPair> {
    EcdsaKeyPair::from_pkcs8(alg.ecdsa_signing(), pkcs8(pem, "EC")?, &SystemRandom::new())
        .map_err(|e| DomainError::new(format!("EC key rejected ({}): {e}", alg.label())))
}

fn ed25519_key_pair(pem: &Pem) -> DomainResult<Ed25519KeyPair> {
    Ed25519KeyPair::from_pkcs8_maybe_unchecked(pkcs8(pem, "Ed25519")?)
        .map_err(|e| DomainError::new(format!("Ed25519 key rejected: {e}")))
}

fn sign_bytes(alg: JwtAlgorithm, key: &str, message: &[u8]) -> DomainResult<Vec<u8>> {
    if alg.is_hmac() {
        let key = hmac::Key::new(alg.hmac(), key.as_bytes());
        return Ok(hmac::sign(&key, message).as_ref().to_vec());
    }
    let pem = parse_pem(key)?;
    let rng = SystemRandom::new();
    let unspecified = |_| DomainError::new("Signing failed.");
    Ok(match alg {
        JwtAlgorithm::Es256 | JwtAlgorithm::Es384 => ecdsa_key_pair(alg, &pem)?
            .sign(&rng, message)
            .map_err(unspecified)?
            .as_ref()
            .to_vec(),
        JwtAlgorithm::EdDsa => ed25519_key_pair(&pem)?.sign(message).as_ref().to_vec(),
        _ => {
            let pair = rsa_key_pair(&pem)?;
            let mut signature = vec![0; pair.public().modulus_len()];
            pair.sign(alg.rsa_padding(), &rng, message, &mut signature)
                .map_err(unspecified)?;
            signature
        }
    })
}

/// Signs a JSON claims object into a compact JWT.
pub fn sign(payload: &str, alg: JwtAlgorithm, key: &str) -> DomainResult<String> {
    let claims: Value = serde_json::from_str(payload)?;
    if !claims.is_object() {
        return Err(DomainError::new("The payload must be a JSON object."));
    }
    if key.is_empty() {
        return Err(DomainError::new(if alg.is_hmac() {
            "Enter the secret."
        } else {
            "Enter the private key (PEM)."
        }));
    }
    let header = json!({ "alg": alg.label(), "typ": "JWT" });
    let signing_input = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?),
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims)?)
    );
    let signature = sign_bytes(alg, key, signing_input.as_bytes())?;
    Ok(format!(
        "{signing_input}.{}",
        URL_SAFE_NO_PAD.encode(signature)
    ))
}

/// Public key bytes for `alg`, from a public key or a private key PEM.
fn public_key(alg: JwtAlgorithm, key: &str) -> DomainResult<Vec<u8>> {
    let pem = parse_pem(key)?;
    let bad_spki = || DomainError::new("Invalid public key (SubjectPublicKeyInfo expected).");
    Ok(match (pem.label.as_str(), alg) {
        ("PUBLIC KEY", _) => spki_key(&pem.der).ok_or_else(bad_spki)?.to_vec(),
        ("RSA PUBLIC KEY", _) => pem.der,
        (_, JwtAlgorithm::Es256 | JwtAlgorithm::Es384) => {
            ecdsa_key_pair(alg, &pem)?.public_key().as_ref().to_vec()
        }
        (_, JwtAlgorithm::EdDsa) => ed25519_key_pair(&pem)?.public_key().as_ref().to_vec(),
        _ => rsa_key_pair(&pem)?.public().as_ref().to_vec(),
    })
}

fn decode_json(part: &str) -> DomainResult<Value> {
    let bytes = URL_SAFE_NO_PAD.decode(part.trim_end_matches('='))?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn time_claim(claims: &Map<String, Value>, name: &str, now: DateTime<Utc>) -> Option<String> {
    let secs = claims.get(name)?.as_i64()?;
    let date = DateTime::from_timestamp(secs, 0)?;
    Some(format!(
        "{} ({})",
        date.format("%Y-%m-%d %H:%M:%S UTC"),
        relative(date, now)
    ))
}

/// Checks a token's signature with `key` (the secret, or a public or private
/// PEM key), then reports its time claims and decoded content.
pub fn verify(token: &str, key: &str, now: DateTime<Utc>) -> DomainResult<String> {
    let parts: Vec<&str> = token.trim().split('.').collect();
    let [header, payload, signature] = parts[..] else {
        return Err(DomainError::new(
            "Not a signed JWT (expected header.payload.signature).",
        ));
    };
    let header_json = decode_json(header)?;
    let payload_json = decode_json(payload)?;
    let alg_label = header_json["alg"].as_str().unwrap_or_default();
    let alg = JwtAlgorithm::from_label(alg_label).ok_or_else(|| {
        DomainError::new(format!(
            "Unsupported or missing \"alg\" in header: {alg_label:?}"
        ))
    })?;
    if key.is_empty() {
        return Err(DomainError::new(if alg.is_hmac() {
            "Enter the secret to verify the signature."
        } else {
            "Enter the public key (PEM) to verify the signature."
        }));
    }
    let message = format!("{header}.{payload}");
    let signature = URL_SAFE_NO_PAD.decode(signature)?;
    let valid = if alg.is_hmac() {
        let key = hmac::Key::new(alg.hmac(), key.as_bytes());
        hmac::verify(&key, message.as_bytes(), &signature).is_ok()
    } else {
        signature::UnparsedPublicKey::new(alg.verification(), public_key(alg, key)?)
            .verify(message.as_bytes(), &signature)
            .is_ok()
    };
    if !valid {
        return Err(DomainError::new(format!(
            "Invalid signature: this token was not signed ({}) with this key.",
            alg.label()
        )));
    }

    let mut lines = vec![(
        "Signature".to_string(),
        format!("✓ valid ({})", alg.label()),
    )];
    if let Some(claims) = payload_json.as_object() {
        for (name, label) in [("iat", "Issued"), ("nbf", "Not before"), ("exp", "Expires")] {
            if let Some(value) = time_claim(claims, name, now) {
                lines.push((label.to_string(), value));
            }
        }
        let expired = claims
            .get("exp")
            .and_then(Value::as_i64)
            .is_some_and(|exp| exp <= now.timestamp());
        let early = claims
            .get("nbf")
            .and_then(Value::as_i64)
            .is_some_and(|nbf| nbf > now.timestamp());
        if expired || early {
            let why = if expired { "expired" } else { "not yet valid" };
            lines.push(("Status".to_string(), format!("⚠ {why}")));
        }
    }
    Ok(format!(
        "{}\n\n{}",
        rows(&lines),
        to_pretty_json(&json!({ "header": header_json, "payload": payload_json }))?
    ))
}

/// Verifies `input` when it is a token, signs it when it is a JSON payload.
pub fn sign_or_verify(
    input: &str,
    alg: JwtAlgorithm,
    key: &str,
    now: DateTime<Utc>,
) -> DomainResult<String> {
    if input.trim_start().starts_with('{') {
        sign(input, alg, key)
    } else {
        verify(input, key, now)
    }
}
