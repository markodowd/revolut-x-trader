use base64::{Engine as _, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signer, SigningKey, pkcs8::DecodePrivateKey as _};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::time::{SystemTime, UNIX_EPOCH};

fn generate_signature(
    private_key: &SigningKey,
    method: &str,
    path: &str,
    query: &str,
    body: &str,
) -> (u128, String) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();

    let message = format!("{}{}{}{}{}", timestamp, method, path, query, body);

    let signature = private_key.sign(message.as_bytes());
    let b64_signature = STANDARD.encode(signature.to_bytes());

    (timestamp, b64_signature)
}

fn select_path() -> &'static str {
    loop {
        println!("1) GET /api/1.0/balances");
        println!("2) GET /api/1.0/configuration/pairs");
        print!("Choice: ");
        io::stdout().flush().expect("flush failed");

        let mut input = String::new();
        io::stdin()
            .lock()
            .read_line(&mut input)
            .expect("read failed");

        match input.trim() {
            "1" => return "/api/1.0/balances",
            "2" => return "/api/1.0/configuration/pairs",
            _ => println!("Invalid choice, try again."),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let api_key = env::var("REVOLUT_X_API_KEY")?;
    let base_url = env::var("REVOLUT_X_BASE_URL")?;

    let pem_content = fs::read_to_string("keys/private.pem")?;
    let der_b64: String = pem_content
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect();
    let der = STANDARD.decode(der_b64)?;
    let signing_key = SigningKey::from_pkcs8_der(&der)?;

    let path = select_path();

    let method = "GET";
    let query = "";
    let body = "";

    let (timestamp, signature) = generate_signature(&signing_key, method, path, query, body);

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    println!("{} {}", method, url);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("X-Revx-API-Key", &api_key)
        .header("X-Revx-Timestamp", timestamp.to_string())
        .header("X-Revx-Signature", &signature)
        .send()?;

    println!("Status: {}", response.status());
    println!("{}", response.text()?);

    Ok(())
}
