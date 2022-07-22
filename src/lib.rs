mod generic_structs;
mod livescore;
mod search;

pub use generic_structs::*;

pub async fn get_all_games() -> Result<Football, Box<dyn std::error::Error>> {
    livescore::get_all_games().await
}
