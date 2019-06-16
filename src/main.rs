use std::io;
use std::io::prelude::*;

fn main() {
    let games = livescore::get_all_games().expect("Main error");
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let query = line.unwrap();
        let games = games.clone();
        let filteredgames = games.query(&query);
        display_football(&filteredgames);
    }
}
fn display_football(football: &livescore::Football) {
    for country in &football.countries {
        println!("{}", country.name);
        for competition in &country.competitions {
            println!(">>> {}", competition.name);
            for game in &competition.games {
                println!(">>>>>> {}", game);
            }
        }
    }
}
