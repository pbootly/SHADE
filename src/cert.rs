use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

pub fn generate_keys() -> Result<(String, String)> {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let priv_b64 = general_purpose::STANDARD.encode(secret.to_bytes());
    let pub_b64 = general_purpose::STANDARD.encode(public.as_bytes());

    Ok((priv_b64, pub_b64))
}
