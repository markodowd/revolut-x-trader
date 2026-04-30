use std::io::{self, BufRead, Write};

pub enum Action {
    Get(&'static str),
    PlaceOrder,
    PlaceSell,
}

pub fn select_action() -> Action {
    loop {
        println!("1) GET /balances");
        println!("2) GET /configuration/pairs");
        println!("3) POST /orders (BUY LTC-USD limit)");
        println!("4) POST /orders (SELL LTC-USD limit)");
        print!("Choice: ");
        io::stdout().flush().expect("flush failed");

        let mut input = String::new();
        io::stdin()
            .lock()
            .read_line(&mut input)
            .expect("read failed");

        match input.trim() {
            "1" => return Action::Get("/balances"),
            "2" => return Action::Get("/configuration/pairs"),
            "3" => return Action::PlaceOrder,
            "4" => return Action::PlaceSell,
            _ => println!("Invalid choice, try again."),
        }
    }
}

pub fn prompt_buy_price(available: &str) -> String {
    println!("Spending {} USD", available);
    prompt("Limit price (USD per LTC): ")
}

pub fn prompt_sell_price(available: &str) -> String {
    println!("Selling {} LTC", available);
    prompt("Limit price (USD per LTC): ")
}

fn prompt(label: &str) -> String {
    loop {
        print!("{}", label);
        io::stdout().flush().expect("flush failed");

        let mut input = String::new();
        io::stdin()
            .lock()
            .read_line(&mut input)
            .expect("read failed");

        let value = input.trim().to_string();
        if !value.is_empty() {
            return value;
        }
        println!("Value cannot be empty.");
    }
}
