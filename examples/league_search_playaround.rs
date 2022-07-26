use football::ranking::beebs::Search;
use std::io;
use std::io::prelude::*;

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=trace cargo run --example thisfilename
    env_logger::init();

    let content = include_str!("../src/ranking/beebs/belgium.1a.html");
    let mut search = Search::new();
    search.update_data(&content).unwrap();
    println!("{:#?}", search);
    let stdin = io::stdin();
    println!("Enter queries");
    for line in stdin.lock().lines() {
        let query = line.unwrap();
        for (score, matched, urls) in search.search(&query) {
            println!("{}: {}. {:?}", score, matched, urls);
        }
    }
}
