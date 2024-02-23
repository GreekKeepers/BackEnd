use blake2::{Blake2b512, Digest};
use hex::ToHex;
use jwt::Error as JwtError;

use crate::jwt::{verify_token, Payload};

pub fn blake_hash(message: &str) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(message.as_bytes());
    hasher.finalize().encode_hex()
}

pub fn serialize_token(input: &str, key: &str) -> Result<Payload, JwtError> {
    verify_token(input, key)
}
