use serde::{Deserialize, Serialize};

const STATE_FILE: &str = "bot_state.json";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Phase {
    Buying,
    Selling,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotState {
    pub cycle: u32,
    pub phase: Phase,
    pub order_id: String,
}

impl BotState {
    pub fn fresh() -> Self {
        Self { cycle: 1, phase: Phase::Buying, order_id: String::new() }
    }
}

pub fn load() -> Option<BotState> {
    std::fs::read_to_string(STATE_FILE)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn save(state: &BotState) {
    if let Ok(json) = serde_json::to_string(state) {
        let _ = std::fs::write(STATE_FILE, json);
    }
}

pub fn clear() {
    let _ = std::fs::remove_file(STATE_FILE);
}
