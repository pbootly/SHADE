use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct RegisterRequest {
    pub public_key: String,
}

