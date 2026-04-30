use std::io::{self, BufRead, Write};

pub enum Action {
    Get(&'static str),
    PlaceOrder { quote_size: String, price: String },
}

pub fn select_action() -> Action {
    loop {
        println!("1) GET /balances");
        println!("2) GET /configuration/pairs");
        println!("3) POST /orders (BUY LTC-USD limit)");
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
            "3" => return prompt_buy_order(),
            _ => println!("Invalid choice, try again."),
        }
    }
}

fn prompt_buy_order() -> Action {
    let quote_size = prompt("USD amount to spend (quote_size): ");
    let price = prompt("Limit price (USD per LTC): ");
    Action::PlaceOrder { quote_size, price }
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
