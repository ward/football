use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::str::FromStr;

pub mod decrypt;

pub fn get_all_games() -> Result<Football, Box<std::error::Error>> {
    let livescore = fetch_livescore()?;
    Ok(parse_livescore(livescore))
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
    result
}

#[derive(Debug)]
pub struct Football {
    countries: Vec<Country>,
}
impl Football {
    // TODO This would be much nicer with on-line approximate string matching.
    //      bitap algorithm or somesuch?
    /// Splits string into pieces, only keeps games for which every piece is matched by either
    /// country, competition, or teams
    pub fn query(self, query: &str) -> Football {
        let query: Vec<_> = query.split(|c: char| !c.is_ascii_alphabetic()).collect();
        let mut games = Football { countries: vec![] };
        for country in &self.countries {
            let mut filteredcompetitions = vec![];
            for competition in &country.competitions {
                let filteredgames: Vec<_> = competition
                    .games
                    .iter()
                    .filter(|game| {
                        query.iter().all(|word| {
                            country.name.contains(word)
                                || competition.name.contains(word)
                                || game.home_team.contains(word)
                                || game.away_team.contains(word)
                        })
                    })
                    .map(|game| game.clone())
                    .collect();
                if filteredgames.len() > 0 {
                    filteredcompetitions.push(Competition {
                        name: competition.name.to_owned(),
                        games: filteredgames,
                    });
                }
            }
            if filteredcompetitions.len() > 0 {
                games.countries.push(Country {
                    name: country.name.to_owned(),
                    competitions: filteredcompetitions,
                });
            }
        }
        games
    }

    // TODO: Time querying (now, today, tomorrow, ended, ...)
}
#[derive(Debug)]
pub struct Country {
    name: String,
    competitions: Vec<Competition>,
}
#[derive(Debug)]
pub struct Competition {
    name: String,
    games: Vec<Game>,
}
#[derive(Debug, Clone)]
pub struct Game {
    home_team: String,
    away_team: String,
    home_score: Option<u8>,
    away_score: Option<u8>,
    status: GameStatus,
}
#[derive(Debug, Clone)]
enum GameStatus {
    Upcoming(u64), // TODO: Replace by Chrono
    Ongoing(String),
    Ended,
    // Other(String),
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
            "FT" | "AET" => Ok(GameStatus::Ended),
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
        let decrypted = read_to_string("src/decrypted.txt");
        let decrypted = decrypted.trim();
        let parsed: LiveScore = serde_json::from_str(&decrypted).unwrap();
        println!("{:#?}", parsed);
        // assert!(false);
    }
}
