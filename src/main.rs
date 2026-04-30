mod api;
mod auth;
mod cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (base_url, signing_key) = auth::init()?;
    let path = cli::select_path();
    api::send_get(&base_url, &signing_key, path)?;
    Ok(())
}
