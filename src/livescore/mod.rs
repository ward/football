use super::Competition;
use super::Country;
use super::Football;
use super::Game;
use super::GameStatus;
use serde::{Deserialize, Serialize};

mod decrypt;

pub fn get_all_games() -> Result<Football, Box<std::error::Error>> {
    let livescore = fetch_livescore()?;
    Ok(parse_livescore(livescore))
}

fn parse_livescore(mut livescore: LiveScore) -> Football {
    let mut result = Football { countries: vec![] };
    if livescore.stages.is_empty() {
        return result;
    }
    livescore.stages.sort_by(|a, b| {
        a.country_name
            .cmp(&b.country_name)
            .then(a.competition_name.cmp(&b.competition_name))
    });
    let mut current_competition = Competition {
        name: livescore.stages[0].competition_name.to_owned(),
        games: vec![],
    };
    let mut current_country = Country {
        name: livescore.stages[0].country_name.to_owned(),
        competitions: vec![],
    };
    for stage in livescore.stages {
        if stage.competition_name != current_competition.name {
            current_country.competitions.push(current_competition);
            current_competition = Competition {
                name: stage.competition_name.to_owned(),
                games: vec![],
            };
        }
        if stage.country_name != current_country.name {
            result.countries.push(current_country);
            current_country = Country {
                name: stage.country_name.to_owned(),
                competitions: vec![],
            };
        }
        for game in stage.games {
            let status: GameStatus = game.time.parse().expect("Game status should always parse");
            let status = status.set_start_time(game.start_time);
            let newgame = Game {
                home_team: game.home[0].name.to_owned(),
                away_team: game.away[0].name.to_owned(),
                home_score: game.home_score.and_then(|s| s.parse().ok()),
                away_score: game.away_score.and_then(|s| s.parse().ok()),
                status: status,
            };
            current_competition.games.push(newgame);
        }
    }
    current_country.competitions.push(current_competition);
    result.countries.push(current_country);
    result
}

fn fetch_livescore() -> Result<LiveScore, Box<std::error::Error>> {
    let utc: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let oneday = chrono::Duration::days(1);
    let yday = format!(
        "http://www.livescore.com/~~/r/07/hpx/soccer/{}/0/",
        (utc - oneday).format("%Y-%m-%d").to_string()
    );
    let tomorrow = format!(
        "http://www.livescore.com/~~/r/07/hpx/soccer/{}/0/",
        (utc + oneday).format("%Y-%m-%d").to_string()
    );
    let urls = vec![
        "http://www.livescore.com/~~/r/07/hp/soccer/0/", // today
        &yday,
        &tomorrow,
    ];
    let mut livescore = LiveScore { stages: vec![] };
    for url in urls {
        livescore.union(fetch_page(url)?);
    }
    Ok(livescore)
}

fn fetch_page(url: &str) -> Result<LiveScore, Box<std::error::Error>> {
    let client = reqwest::Client::new();
    let builder = client.get(url).header(
        reqwest::header::USER_AGENT,
        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:68.0) Gecko/20100101 Firefox/68.0",
    );
    let encrypted: String = builder.send()?.text()?;
    let decrypted = decrypt::decrypt(&encrypted);
    let parsed = serde_json::from_str(&decrypted)?;
    Ok(parsed)
}

#[derive(Serialize, Deserialize, Debug)]
struct LiveScore {
    #[serde(rename = "Stages")]
    stages: Vec<LiveScoreStage>,
}
impl LiveScore {
    fn union(&mut self, mut other: LiveScore) {
        self.stages.append(&mut other.stages);
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct LiveScoreStage {
    #[serde(rename = "Cnm")]
    country_name: String,
    #[serde(rename = "Snm")]
    competition_name: String,
    // default catches situations where there is no "Events"
    #[serde(rename = "Events", default)]
    games: Vec<LiveScoreGames>,
}
#[derive(Serialize, Deserialize, Debug)]
struct LiveScoreGames {
    #[serde(rename = "Eps")]
    time: String,
    #[serde(rename = "Esd")]
    start_time: u64,
    #[serde(rename = "T1")]
    home: Vec<LiveScoreTeam>,
    #[serde(rename = "T2")]
    away: Vec<LiveScoreTeam>,
    #[serde(rename = "Tr1")]
    home_score: Option<String>,
    #[serde(rename = "Tr2")]
    away_score: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct LiveScoreTeam {
    #[serde(rename = "Nm")]
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_to_string(path: &str) -> String {
        use std::fs::File;
        use std::io::Read;

        let mut f = File::open(path).unwrap();
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        buffer
    }

    #[test]
    fn parse_decrypted() {
        let decrypted = read_to_string("src/livescore/decrypted.txt");
        let decrypted = decrypted.trim();
        let parsed: LiveScore = serde_json::from_str(&decrypted).unwrap();
        println!("{:#?}", parsed);
        // assert!(false);
    }

    #[test]
    fn query_games() {
        let decrypted = read_to_string("src/livescore/decrypted.txt");
        let decrypted = decrypted.trim();
        let parsed: LiveScore = serde_json::from_str(&decrypted).unwrap();
        let games = parse_livescore(parsed);
        println!("{:#?}", games);
        let euro_spain = games.query("euro spain");
        println!("{:#?}", euro_spain);
        assert_eq!(euro_spain.number_of_games(), 1);
    }
}