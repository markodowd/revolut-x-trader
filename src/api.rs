use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::auth::sign_request;

fn url_path(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    trimmed
        .splitn(2, "://")
        .nth(1)
        .and_then(|s| s.find('/').map(|i| s[i..].to_string()))
        .unwrap_or_default()
}

pub fn send_get(
    base_url: &str,
    signing_key: &SigningKey,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let sign_path = format!("{}{}", url_path(base_url), path);
    let (api_key, timestamp, signature) = sign_request(signing_key, "GET", &sign_path, "", "")?;

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

    let output = if path == "/balances" {
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

pub fn get_available(
    base_url: &str,
    signing_key: &SigningKey,
    currency: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let path = "/balances";
    let sign_path = format!("{}{}", url_path(base_url), path);
    let (api_key, timestamp, signature) = sign_request(signing_key, "GET", &sign_path, "", "")?;

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    let body: serde_json::Value = reqwest::blocking::Client::new()
        .get(&url)
        .header("Accept", "application/json")
        .header("X-Revx-API-Key", &api_key)
        .header("X-Revx-Timestamp", timestamp.to_string())
        .header("X-Revx-Signature", &signature)
        .send()?
        .json()?;

    body.as_array()
        .and_then(|arr| arr.iter().find(|item| item["currency"] == currency))
        .and_then(|entry| entry["available"].as_str().map(|s| s.to_string()))
        .ok_or_else(|| format!("{} available balance not found", currency).into())
}

pub fn cancel_all_orders(
    base_url: &str,
    signing_key: &SigningKey,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = "/orders";
    let sign_path = format!("{}{}", url_path(base_url), path);
    let (api_key, timestamp, signature) = sign_request(signing_key, "DELETE", &sign_path, "", "")?;

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    println!("DELETE {}", url);

    let response = reqwest::blocking::Client::new()
        .delete(&url)
        .header("X-Revx-API-Key", &api_key)
        .header("X-Revx-Timestamp", timestamp.to_string())
        .header("X-Revx-Signature", &signature)
        .send()?;

    let status = response.status();
    println!("Status: {}", status);

    if !status.is_success() {
        let body = response.text().unwrap_or_else(|_| "<unreadable body>".to_string());
        println!("Error body: {}", body);
        return Err(format!("Cancel all orders failed with status {}", status).into());
    }

    println!("All active orders cancelled.");
    Ok(())
}

// --- Place Order ---

#[derive(Serialize)]
struct PlaceOrderRequest {
    client_order_id: String,
    symbol: String,
    side: String,
    order_configuration: OrderConfiguration,
}

#[derive(Serialize)]
struct OrderConfiguration {
    limit: LimitConfig,
}

#[derive(Serialize)]
struct LimitConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    quote_size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_size: Option<String>,
    price: String,
    execution_instructions: Vec<String>,
}

#[derive(Deserialize)]
struct PlaceOrderResponse {
    data: OrderData,
}

#[derive(Deserialize)]
struct OrderData {
    venue_order_id: String,
    client_order_id: String,
    state: String,
}

pub fn place_order(
    base_url: &str,
    signing_key: &SigningKey,
    side: &str,
    size: &str,
    price: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = "/orders";

    let limit = if side == "buy" {
        LimitConfig {
            quote_size: Some(size.to_string()),
            base_size: None,
            price: price.to_string(),
            execution_instructions: vec!["allow_taker".to_string()],
        }
    } else {
        LimitConfig {
            quote_size: None,
            base_size: Some(size.to_string()),
            price: price.to_string(),
            execution_instructions: vec!["allow_taker".to_string()],
        }
    };

    let order = PlaceOrderRequest {
        client_order_id: Uuid::new_v4().to_string(),
        symbol: "LTC-USD".to_string(),
        side: side.to_string(),
        order_configuration: OrderConfiguration { limit },
    };

    let body = serde_json::to_string(&order)?;
    let sign_path = format!("{}{}", url_path(base_url), path);
    let (api_key, timestamp, signature) = sign_request(signing_key, "POST", &sign_path, "", &body)?;

    let url = format!("{}{}", base_url.trim_end_matches('/'), path);
    println!("POST {}", url);

    let response = reqwest::blocking::Client::new()
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("X-Revx-API-Key", &api_key)
        .header("X-Revx-Timestamp", timestamp.to_string())
        .header("X-Revx-Signature", &signature)
        .body(body)
        .send()?;

    let status = response.status();
    println!("Status: {}", status);

    if !status.is_success() {
        let body = response.text().unwrap_or_else(|_| "<unreadable body>".to_string());
        println!("Error body: {}", body);
        return Err(format!("Order request failed with status {}", status).into());
    }

    let order_response: PlaceOrderResponse = response.json()?;
    println!("venue_order_id:  {}", order_response.data.venue_order_id);
    println!("client_order_id: {}", order_response.data.client_order_id);
    println!("state:           {}", order_response.data.state);

    Ok(())
}
