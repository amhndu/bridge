use quirc::client::*;

fn main() {
    println!("Hello, world!");

    let _client = Client::connect(String::from("localhost:6667"), String::from("quirc-bot"));
}
