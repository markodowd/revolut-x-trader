use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey};
use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_signature(
    private_key: &SigningKey,
    method: &str,
    path: &str,
    query: &str,
    body: &str,
) -> (String, String) {
    // 1. Generate Unix timestamp in milliseconds
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string();

    // 2. Construct the message exactly as the API expects
    let message = format!("{}{}{}{}{}", timestamp, method, path, query, body);

    // 3. Sign and Base64 encode
    let signature = private_key.sign(message.as_bytes());
    let b64_signature = STANDARD.encode(signature.to_bytes());

    (timestamp, b64_signature)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let api_key = env::var("REVOLUT_X_API_KEY")?;

    // Load your key (do this once, not for every request)
    let pem_content = fs::read_to_string("keys/private.pem")?;
    let (_, doc) = pkcs8::SecretDocument::from_pem(&pem_content)?;
    let key_bytes: &[u8; 32] = doc.as_bytes()[..32].try_into()?;
    let signing_key = SigningKey::from_bytes(key_bytes);

    // Example Usage
    let (ts, sig) = generate_signature(
        &signing_key,
        "GET",
        "/api/1.0/orders/active",
        "status=open&limit=10",
        "",
    );

    println!("X-Revx-Timestamp: {}", ts);
    println!("X-Revx-Signature: {}", sig);
    println!("X-Revx-Api-Key: {}", api_key);

    Ok(())
}
