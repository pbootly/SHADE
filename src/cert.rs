use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

pub fn generate_keys() -> Result<(String, String)> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let priv_b64 = general_purpose::STANDARD.encode(secret.to_bytes());
    let pub_b64 = generate_public_from_private(&priv_b64)?;

    Ok((priv_b64, pub_b64))
}

pub fn generate_public_from_private(priv_b64: &str) -> Result<String> {
    let priv_bytes = base64::engine::general_purpose::STANDARD.decode(priv_b64.trim())?;
    let secret = StaticSecret::from(<[u8; 32]>::try_from(priv_bytes.as_slice())?);
    let public = PublicKey::from(&secret);
    let pub_b64 = general_purpose::STANDARD.encode(public.as_bytes());
    Ok(pub_b64)
}
