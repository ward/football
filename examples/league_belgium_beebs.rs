use football::ranking::beebs::Beebs;

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=trace cargo run --example thisfilename
    env_logger::init();

    let beebs = Beebs::new().await;
    println!("{:#?}", beebs);
}
