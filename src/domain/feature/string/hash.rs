use sha2::Digest;

use crate::domain::entity::feature_entity::FeatureEntity;

pub const ENTITY: FeatureEntity = FeatureEntity {
    name: "hash",
    section: "string",
    title: "Hash Generator",
    description: "Generate cryptographic hash digests",
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Blake2b,
}

impl HashAlgorithm {
    pub const ALL: [HashAlgorithm; 8] = [
        HashAlgorithm::Md5,
        HashAlgorithm::Sha1,
        HashAlgorithm::Sha224,
        HashAlgorithm::Sha256,
        HashAlgorithm::Sha384,
        HashAlgorithm::Sha512,
        HashAlgorithm::Sha3_256,
        HashAlgorithm::Blake2b,
    ];
    pub const DEFAULT: HashAlgorithm = HashAlgorithm::Sha256;

    pub fn label(self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "md5",
            HashAlgorithm::Sha1 => "sha1",
            HashAlgorithm::Sha224 => "sha224",
            HashAlgorithm::Sha256 => "sha256",
            HashAlgorithm::Sha384 => "sha384",
            HashAlgorithm::Sha512 => "sha512",
            HashAlgorithm::Sha3_256 => "sha3_256",
            HashAlgorithm::Blake2b => "blake2b",
        }
    }
}

fn hex<D: Digest>(data: &[u8]) -> String {
    D::digest(data).iter().map(|b| format!("{b:02x}")).collect()
}

/// Lower-case hexadecimal digest of `data`.
pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> String {
    match algorithm {
        HashAlgorithm::Md5 => hex::<md5::Md5>(data),
        HashAlgorithm::Sha1 => hex::<sha1::Sha1>(data),
        HashAlgorithm::Sha224 => hex::<sha2::Sha224>(data),
        HashAlgorithm::Sha256 => hex::<sha2::Sha256>(data),
        HashAlgorithm::Sha384 => hex::<sha2::Sha384>(data),
        HashAlgorithm::Sha512 => hex::<sha2::Sha512>(data),
        HashAlgorithm::Sha3_256 => hex::<sha3::Sha3_256>(data),
        HashAlgorithm::Blake2b => hex::<blake2::Blake2b512>(data),
    }
}
