use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::auth::sign_request;

fn url_path(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    trimmed
        .splitn(2, "://")
        .nth(1)
        .and_then(|s| s.find('/').map(|i| s[i..].to_string()))
        .unwrap_or_default()
}

fn send_signed(
    base_url: &str,
    signing_key: &SigningKey,
    method: &str,
    path: &str,
    body: &str,
) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
    let sign_path = format!("{}{}", url_path(base_url), path);
    let (api_key, timestamp, signature) = sign_request(signing_key, method, &sign_path, "", body)?;
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    let client = reqwest::blocking::Client::new();
    let mut builder = match method {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "DELETE" => client.delete(&url),
        _ => return Err(format!("unsupported HTTP method: {}", method).into()),
    }
    .header("Accept", "application/json")
    .header("X-Revx-API-Key", &api_key)
    .header("X-Revx-Timestamp", timestamp.to_string())
    .header("X-Revx-Signature", &signature);

    if !body.is_empty() {
        builder = builder
            .header("Content-Type", "application/json")
            .body(body.to_string());
    }

    Ok(builder.send()?)
}

fn require_success(
    response: reqwest::blocking::Response,
    context: &str,
) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_else(|_| "<unreadable body>".to_string());
        println!("Error body: {}", body);
        return Err(format!("{} failed with status {}", context, status).into());
    }
    Ok(response)
}

pub fn send_get(
    base_url: &str,
    signing_key: &SigningKey,
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("GET {}{}", base_url.trim_end_matches('/'), path);
    let response = send_signed(base_url, signing_key, "GET", path, "")?;
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
    let body: serde_json::Value = send_signed(base_url, signing_key, "GET", "/balances", "")?.json()?;

    body.as_array()
        .and_then(|arr| arr.iter().find(|item| item["currency"] == currency))
        .and_then(|entry| entry["available"].as_str().map(|s| s.to_string()))
        .ok_or_else(|| format!("{} available balance not found", currency).into())
}

pub fn cancel_all_orders(
    base_url: &str,
    signing_key: &SigningKey,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("DELETE {}/orders", base_url.trim_end_matches('/'));
    let response = send_signed(base_url, signing_key, "DELETE", "/orders", "")?;
    println!("Status: {}", response.status());
    require_success(response, "Cancel all orders")?;
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

pub fn get_active_order_ids(
    base_url: &str,
    signing_key: &SigningKey,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let body: serde_json::Value = send_signed(base_url, signing_key, "GET", "/orders/active", "")?.json()?;

    let orders = body.as_array()
        .or_else(|| body.get("data").and_then(|d| d.as_array()));

    Ok(orders
        .map(|arr| {
            arr.iter()
                .filter_map(|item| item["client_order_id"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default())
}

pub fn place_order(
    base_url: &str,
    signing_key: &SigningKey,
    side: &str,
    size: &str,
    price: &str,
    client_order_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let limit = LimitConfig {
        quote_size: (side == "buy").then(|| size.to_string()),
        base_size: (side != "buy").then(|| size.to_string()),
        price: price.to_string(),
        execution_instructions: vec!["allow_taker".to_string()],
    };

    let order = PlaceOrderRequest {
        client_order_id: client_order_id.to_string(),
        symbol: "LTC-USD".to_string(),
        side: side.to_string(),
        order_configuration: OrderConfiguration { limit },
    };

    let body = serde_json::to_string(&order)?;
    println!("POST {}/orders", base_url.trim_end_matches('/'));
    let response = send_signed(base_url, signing_key, "POST", "/orders", &body)?;
    println!("Status: {}", response.status());
    let response = require_success(response, "Order request")?;
    let order_response: PlaceOrderResponse = response.json()?;
    println!("venue_order_id:  {}", order_response.data.venue_order_id);
    println!("client_order_id: {}", order_response.data.client_order_id);
    println!("state:           {}", order_response.data.state);

    Ok(order_response.data.client_order_id)
}
