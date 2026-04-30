use ed25519_dalek::SigningKey;
use std::fs;

use crate::auth::sign_request;

pub fn send_get(
    base_url: &str,
    signing_key: &SigningKey,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (api_key, timestamp, signature) = sign_request(signing_key, "GET", path, "", "")?;

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    println!("GET {}", url);

    let response = reqwest::blocking::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .header("X-Revx-API-Key", &api_key)
        .header("X-Revx-Timestamp", timestamp.to_string())
        .header("X-Revx-Signature", &signature)
        .send()?;

    println!("Status: {}", response.status());
    let body: serde_json::Value = response.json()?;
    let pretty = serde_json::to_string_pretty(&body)?;
    println!("{}", pretty);
    fs::write("output.txt", &pretty)?;

    Ok(())
}
