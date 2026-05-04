mod api;
mod auth;
mod cli;
mod state;

use ed25519_dalek::SigningKey;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};
use uuid::Uuid;

fn with_retry<T, F>(mut op: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnMut() -> Result<T, Box<dyn std::error::Error>>,
{
    let mut delay = Duration::from_secs(2);
    for attempt in 1..=3u32 {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if attempt < 3 => {
                println!("Attempt {} failed: {}. Retrying in {:?}...", attempt, e, delay);
                std::thread::sleep(delay);
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

// Returns Ok(true) when the order is filled, Ok(false) when shutdown is requested.
fn poll_until_filled(
    base_url: &str,
    signing_key: &SigningKey,
    order_id: &str,
    shutdown: &Arc<AtomicBool>,
) -> Result<bool, Box<dyn std::error::Error>> {
    loop {
        // Check immediately before sleeping so a resume after a fill is instant.
        let active = with_retry(|| api::get_active_order_ids(base_url, signing_key))?;
        if !active.contains(&order_id.to_string()) {
            return Ok(true);
        }
        println!("Order still open. Next check in 5 min...");

        // Sleep in 10s chunks so Ctrl+C is noticed within ~10 seconds.
        let deadline = Instant::now() + Duration::from_secs(300);
        while Instant::now() < deadline {
            if shutdown.load(Ordering::SeqCst) {
                return Ok(false);
            }
            std::thread::sleep(Duration::from_secs(10));
        }

        if shutdown.load(Ordering::SeqCst) {
            return Ok(false);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (base_url, signing_key) = auth::init()?;

    match cli::select_action() {
        cli::Action::Get(path) => api::send_get(&base_url, &signing_key, path)?,
        cli::Action::PlaceOrder => {
            let size = api::get_available(&base_url, &signing_key, "USD")?;
            let price = cli::prompt_buy_price(&size);
            api::place_order(&base_url, &signing_key, "buy", &size, &price, &Uuid::new_v4().to_string())?;
        }
        cli::Action::PlaceSell => {
            let size = api::get_available(&base_url, &signing_key, "LTC")?;
            let price = cli::prompt_sell_price(&size);
            api::place_order(&base_url, &signing_key, "sell", &size, &price, &Uuid::new_v4().to_string())?;
        }
        cli::Action::CancelAllOrders => api::cancel_all_orders(&base_url, &signing_key)?,
        cli::Action::Bot { buy_price, sell_price } => {
            let shutdown = Arc::new(AtomicBool::new(false));
            {
                let s = Arc::clone(&shutdown);
                ctrlc::set_handler(move || {
                    println!("\nShutdown requested. Will stop after current poll.");
                    s.store(true, Ordering::SeqCst);
                })?;
            }

            let mut bot = match state::load() {
                Some(s) => {
                    println!("Resuming saved state: cycle {}, phase {:?}, order {:?}", s.cycle, s.phase, s.order_id);
                    s
                }
                None => state::BotState::fresh(),
            };

            'bot: loop {
                println!("\n=== Cycle {} ===", bot.cycle);

                // BUY PHASE
                if bot.phase == state::Phase::Buying {
                    if bot.order_id.is_empty() {
                        let size = with_retry(|| api::get_available(&base_url, &signing_key, "USD"))?;
                        let order_id = Uuid::new_v4().to_string();
                        println!("Placing buy: {} USD at ${}", size, buy_price);
                        with_retry(|| api::place_order(&base_url, &signing_key, "buy", &size, &buy_price, &order_id))?;
                        bot.order_id = order_id;
                        state::save(&bot);
                    }

                    let filled = poll_until_filled(&base_url, &signing_key, &bot.order_id, &shutdown)?;
                    if !filled {
                        println!("Shutting down. Cancelling all orders...");
                        let _ = api::cancel_all_orders(&base_url, &signing_key);
                        state::clear();
                        break 'bot;
                    }
                    println!("Buy filled.");
                    bot.phase = state::Phase::Selling;
                    bot.order_id = String::new();
                    state::save(&bot);
                }

                // SELL PHASE
                if bot.order_id.is_empty() {
                    let size = with_retry(|| api::get_available(&base_url, &signing_key, "LTC"))?;
                    let order_id = Uuid::new_v4().to_string();
                    println!("Placing sell: {} LTC at ${}", size, sell_price);
                    with_retry(|| api::place_order(&base_url, &signing_key, "sell", &size, &sell_price, &order_id))?;
                    bot.order_id = order_id;
                    state::save(&bot);
                }

                let filled = poll_until_filled(&base_url, &signing_key, &bot.order_id, &shutdown)?;
                if !filled {
                    println!("Shutting down. Cancelling all orders...");
                    let _ = api::cancel_all_orders(&base_url, &signing_key);
                    state::clear();
                    break 'bot;
                }
                println!("Sell filled. Restarting cycle.");
                bot.cycle += 1;
                bot.phase = state::Phase::Buying;
                bot.order_id = String::new();
                state::save(&bot);
            }
        }
    }

    Ok(())
}
