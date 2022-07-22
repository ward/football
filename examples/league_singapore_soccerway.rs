//! Example of fetching some data and showing the raw result.

use football::ranking::soccerway;

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=trace cargo run --example singapore_soccerway
    env_logger::init();

    let mut singapore = soccerway::League::new(String::from(
        "https://int.soccerway.com/national/singapore/sleague/2022/regular-season/r66125/",
    ));
    singapore.update().await.unwrap();
    println!("{:#?}", singapore);
}
