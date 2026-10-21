use dupot_dev_tools::domain::feature::string::hash::{HashAlgorithm, hash};

#[test]
fn known_digests_of_abc() {
    let cases = [
        (HashAlgorithm::Md5, "900150983cd24fb0d6963f7d28e17f72"),
        (
            HashAlgorithm::Sha1,
            "a9993e364706816aba3e25717850c26c9cd0d89d",
        ),
        (
            HashAlgorithm::Sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            HashAlgorithm::Sha3_256,
            "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532",
        ),
    ];
    for (algorithm, expected) in cases {
        assert_eq!(hash(algorithm, b"abc"), expected, "{}", algorithm.label());
    }
}

#[test]
fn digest_lengths() {
    let lengths = [56, 96, 128, 128];
    let algorithms = [
        HashAlgorithm::Sha224,
        HashAlgorithm::Sha384,
        HashAlgorithm::Sha512,
        HashAlgorithm::Blake2b,
    ];
    for (algorithm, len) in algorithms.into_iter().zip(lengths) {
        assert_eq!(hash(algorithm, b"").len(), len, "{}", algorithm.label());
    }
}

#[test]
fn default_is_sha256_and_listed() {
    assert_eq!(HashAlgorithm::DEFAULT.label(), "sha256");
    assert!(HashAlgorithm::ALL.contains(&HashAlgorithm::DEFAULT));
}
