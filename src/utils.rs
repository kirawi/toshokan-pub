use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bincode::{config::standard, decode_from_slice, encode_to_vec, Decode, Encode};

pub fn encode_bincode<E: Encode>(e: &E) -> Result<Vec<u8>> {
    let config = standard();
    let res = encode_to_vec(e, config)?;
    Ok(res)
}

pub fn decode_bincode<D: Decode>(v: &[u8]) -> Result<D> {
    let config = standard();
    let res = decode_from_slice(v, config)?.0;
    Ok(res)
}

pub fn b64_decode_uuid(s: &str) -> Vec<u8> {
    URL_SAFE_NO_PAD.decode(s).unwrap()
}

pub fn b64_encode_uuid(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}
