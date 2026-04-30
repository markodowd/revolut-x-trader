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

    let output = if path == "/api/1.0/balances" {
        let filtered: Vec<&serde_json::Value> = body
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|item| matches!(item["currency"].as_str(), Some("USD") | Some("LTC")))
                    .collect()
            })
            .unwrap_or_default();
        serde_json::to_string_pretty(&filtered)?
    } else {
        serde_json::to_string_pretty(&body)?
    };

    println!("{}", output);
    fs::write("output.txt", &output)?;

    Ok(())
}
