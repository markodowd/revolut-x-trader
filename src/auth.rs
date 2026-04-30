use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey, pkcs8::DecodePrivateKey as _};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn init() -> Result<(String, SigningKey), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let base_url = env::var("REVOLUT_X_BASE_URL")?;

    let pem_content = fs::read_to_string("keys/private.pem")?;
    let der_b64: String = pem_content
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();
    let der = STANDARD.decode(der_b64)?;
    let signing_key = SigningKey::from_pkcs8_der(&der)?;

    Ok((base_url, signing_key))
}

pub fn sign_request(
    private_key: &SigningKey,
    method: &str,
    path: &str,
    query: &str,
    body: &str,
) -> Result<(String, u128, String), Box<dyn std::error::Error>> {
    let api_key = env::var("REVOLUT_X_API_KEY")?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let message = format!("{}{}{}{}{}", timestamp, method, path, query, body);
    let signature = private_key.sign(message.as_bytes());
    let b64_signature = STANDARD.encode(signature.to_bytes());

    Ok((api_key, timestamp, b64_signature))
}
