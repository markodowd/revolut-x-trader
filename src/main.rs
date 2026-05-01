mod api;
mod auth;
mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (base_url, signing_key) = auth::init()?;

    match cli::select_action() {
        cli::Action::Get(path) => api::send_get(&base_url, &signing_key, path)?,
        cli::Action::PlaceOrder => {
            let size = api::get_available(&base_url, &signing_key, "USD")?;
            let price = cli::prompt_buy_price(&size);
            api::place_order(&base_url, &signing_key, "buy", &size, &price)?;
        }
        cli::Action::PlaceSell => {
            let size = api::get_available(&base_url, &signing_key, "LTC")?;
            let price = cli::prompt_sell_price(&size);
            api::place_order(&base_url, &signing_key, "sell", &size, &price)?;
        }
        cli::Action::CancelAllOrders => api::cancel_all_orders(&base_url, &signing_key)?,
        cli::Action::Bot { buy_price, sell_price } => {
            let mut cycle = 1u32;
            loop {
                println!("\n=== Cycle {} ===", cycle);

                // Place buy order
                let size = api::get_available(&base_url, &signing_key, "USD")?;
                println!("Placing buy: {} USD at ${}", size, buy_price);
                let buy_id = api::place_order(&base_url, &signing_key, "buy", &size, &buy_price)?;

                // Poll until buy is filled
                loop {
                    println!("Sleeping 1 hour...");
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                    let active = api::get_active_order_ids(&base_url, &signing_key)?;
                    if active.contains(&buy_id) {
                        println!("Buy order still open.");
                    } else {
                        println!("Buy order filled.");
                        break;
                    }
                }

                // Place sell order
                let size = api::get_available(&base_url, &signing_key, "LTC")?;
                println!("Placing sell: {} LTC at ${}", size, sell_price);
                let sell_id = api::place_order(&base_url, &signing_key, "sell", &size, &sell_price)?;

                // Poll until sell is filled
                loop {
                    println!("Sleeping 1 hour...");
                    std::thread::sleep(std::time::Duration::from_secs(3600));
                    let active = api::get_active_order_ids(&base_url, &signing_key)?;
                    if active.contains(&sell_id) {
                        println!("Sell order still open.");
                    } else {
                        println!("Sell order filled. Restarting cycle.");
                        break;
                    }
                }

                cycle += 1;
            }
        }
    }

    Ok(())
}
