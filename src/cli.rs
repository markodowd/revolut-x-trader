use std::io::{self, BufRead, Write};

pub fn select_path() -> &'static str {
    loop {
        println!("1) GET /api/1.0/balances");
        println!("2) GET /api/1.0/configuration/pairs");
        print!("Choice: ");
        io::stdout().flush().expect("flush failed");

        let mut input = String::new();
        io::stdin()
            .lock()
            .read_line(&mut input)
            .expect("read failed");

        match input.trim() {
            "1" => return "/api/1.0/balances",
            "2" => return "/api/1.0/configuration/pairs",
            _ => println!("Invalid choice, try again."),
        }
    }
}
