mod api;
mod auth;
mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (base_url, signing_key) = auth::init()?;

    match cli::select_action() {
        cli::Action::Get(path) => api::send_get(&base_url, &signing_key, path)?,
        cli::Action::PlaceOrder => {
            let quote_size = api::get_usd_available(&base_url, &signing_key)?;
            let price = cli::prompt_price(&quote_size);
            api::place_order(&base_url, &signing_key, &quote_size, &price)?
        }
    }

    Ok(())
}
