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
            api::place_order(&base_url, &signing_key, "buy", &size, &price)?
        }
        cli::Action::PlaceSell => {
            let size = api::get_available(&base_url, &signing_key, "LTC")?;
            let price = cli::prompt_sell_price(&size);
            api::place_order(&base_url, &signing_key, "sell", &size, &price)?
        }
        cli::Action::CancelAllOrders => api::cancel_all_orders(&base_url, &signing_key)?,
    }

    Ok(())
}
