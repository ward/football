use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::str::FromStr;

pub mod decrypt;

pub fn fetch_homepage() -> Result<(), Box<std::error::Error>> {
    let client = reqwest::Client::new();
    let builder = client
        .get("http://www.livescore.com/~~/r/06/hp/soccer/0/")
        .header(
            reqwest::header::USER_AGENT,
            "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:68.0) Gecko/20100101 Firefox/68.0",
        );
    let encrypted: String = builder.send()?.text()?;
    let decrypted = decrypt::decrypt(&encrypted);
    println!("{:#?}", parse_livescore(&decrypted));
    Ok(())
}

fn parse_livescore(input: &str) -> Vec<Country> {
    let mut parsed: LiveScore = serde_json::from_str(input).unwrap();
    let mut result = vec![];
    if parsed.stages.is_empty() {
        return result;
    }
    parsed.stages.sort_by(|a, b| {
        a.country_name
            .cmp(&b.country_name)
            .then(a.competition_name.cmp(&b.competition_name))
    });
    let mut current_competition = Competition {
        name: parsed.stages[0].competition_name.to_owned(),
        games: vec![],
    };
    let mut current_country = Country {
        name: parsed.stages[0].country_name.to_owned(),
        competitions: vec![],
    };
    for stage in parsed.stages {
        if stage.competition_name != current_competition.name {
            current_country.competitions.push(current_competition);
            current_competition = Competition {
                name: stage.competition_name.to_owned(),
                games: vec![],
            };
        }
        if stage.country_name != current_country.name {
            result.push(current_country);
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
    result
}

#[derive(Debug)]
struct Country {
    name: String,
    competitions: Vec<Competition>,
}
#[derive(Debug)]
struct Competition {
    name: String,
    games: Vec<Game>,
}
#[derive(Debug)]
struct Game {
    home_team: String,
    away_team: String,
    home_score: Option<u8>,
    away_score: Option<u8>,
    status: GameStatus,
}
#[derive(Debug)]
enum GameStatus {
    Upcoming(u64), // TODO: Replace by Chrono
    Ongoing(String),
    Ended,
    Other(String),
}
impl GameStatus {
    fn set_start_time(self, start_time: u64) -> GameStatus {
        if let GameStatus::Upcoming(_) = self {
            GameStatus::Upcoming(start_time)
        } else {
            self
        }
    }
}
impl FromStr for GameStatus {
    type Err = ParseGameStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // TODO Can we use start_time immediately?
            "NS" => Ok(GameStatus::Upcoming(0)),
            "FT" => Ok(GameStatus::Ended),
            // TODO: Only want this for in game time indications (Minutes + HT + ???)
            t => Ok(GameStatus::Ongoing(t.to_owned())),
            // _ => Err(ParseClubLeaderboardSortError),
        }
    }
}
#[derive(Debug, Clone)]
struct ParseGameStatusError;
impl error::Error for ParseGameStatusError {
    fn description(&self) -> &str {
        "Failed to parse game status."
    }
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
impl fmt::Display for ParseGameStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse game status.")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct LiveScore {
    #[serde(rename = "Stages")]
    stages: Vec<LiveScoreStage>,
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
    fn run_fetch() {
        let decrypted = read_to_string("src/decrypted.txt");
        let decrypted = decrypted.trim();
        let parsed = parse_livescore(&decrypted);
        println!("{:#?}", parsed);
        assert!(false);
    }
}
