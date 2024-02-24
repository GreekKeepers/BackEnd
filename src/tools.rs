use blake2::{Blake2b512, Blake2s256, Digest};
use hex::ToHex;
use jwt::Error as JwtError;

use crate::jwt::{verify_token, Payload};

pub fn blake_hash(message: &str) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(message.as_bytes());
    hasher.finalize().encode_hex()
}

pub fn blake_hash_256(message: &str) -> String {
    let mut hasher = Blake2s256::new();
    hasher.update(message.as_bytes());
    hasher.finalize().encode_hex()
}

pub fn blake_hash_256_u64(message: &str) -> u64 {
    let mut hasher = Blake2s256::new();
    hasher.update(message.as_bytes());
    let res: Vec<u8> = hasher.finalize().into_iter().collect();

    (res[0] as u64) << 56
        | (res[1] as u64) << 48
        | (res[2] as u64) << 40
        | (res[3] as u64) << 32
        | (res[4] as u64) << 24
        | (res[5] as u64) << 16
        | (res[6] as u64) << 8
        | (res[7] as u64)
}

pub fn serialize_token(input: &str, key: &str) -> Result<Payload, JwtError> {
    verify_token(input, key)
}
