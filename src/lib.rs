mod generic_structs;
mod livescore;
mod search;

pub use generic_structs::*;

pub fn get_all_games() -> Result<Football, Box<dyn std::error::Error>> {
    livescore::get_all_games()
}
