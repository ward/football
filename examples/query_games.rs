use football::Football;
use std::io;
use std::io::prelude::*;

#[tokio::main]
async fn main() {
    let games = football::get_all_games().await.expect("Main error");
    let stdin = io::stdin();
    println!("Enter queries");
    for line in stdin.lock().lines() {
        let query = line.unwrap();
        let games = games.clone();
        let filteredgames = games.query(&query);
        // let filteredgames = games.mixed_query(&query);
        // for (score, country, competition, game) in filteredgames {
        //     println!("{} {} {} {}", score, country.name, competition.name, game);
        // }
        // let filteredgames = match query.as_str() {
        //     "today" => games.today(),
        //     "tomorrow" => games.tomorrow(),
        //     "yesterday" => games.yesterday(),
        //     "ended" => games.ended(),
        //     "live" => games.live(),
        //     "upcoming" => games.upcoming(),
        //     _ => games.query(&query),
        // };
        // let filteredgames = games.competition(&query);
        _display_football(&filteredgames);
    }
}
fn _display_football(football: &Football) {
    for country in &football.countries {
        println!("{}", country.name);
        for competition in &country.competitions {
            println!(">>> {}", competition.name);
            for game in &competition.games {
                println!(">>>>>> {} {}", game.start_time, game);
            }
        }
    }
}
