use super::Competition;
use super::Country;
use super::Football;
use super::Game;
use super::GameStatus;
use serde::{Deserialize, Serialize};
// Need this for datetime_from_str
use chrono::prelude::*;

// mod decrypt;

pub async fn get_all_games() -> Result<Football, Box<dyn std::error::Error>> {
    let livescore = fetch_livescore().await?;
    Ok(parse_livescore(livescore))
}

fn parse_livescore(mut livescore: LiveScore) -> Football {
    let mut result = Football { countries: vec![] };
    if livescore.stages.is_empty() {
        return result;
    }
    // We do the sorting because fetching multiple days puts things in inconsistent order across
    // days.
    livescore.sort_by_priority();
    let mut current_competition = Competition {
        name: livescore.stages[0].competition_name.to_owned(),
        games: vec![],
    };
    let mut current_country = Country {
        name: livescore.stages[0].country_name.to_owned(),
        competitions: vec![],
    };
    for stage in livescore.stages {
        // If competition or country are called differently, we're in a new competition. Save
        // previous, make new one.
        // Country renewal handled in next if.
        if stage.competition_name != current_competition.name
            || stage.country_name != current_country.name
        {
            current_country.competitions.push(current_competition);
            current_competition = Competition {
                name: stage.competition_name.to_owned(),
                games: vec![],
            };
        }
        // If country is called differently, we're in a new country. Save previous, make new one.
        if stage.country_name != current_country.name {
            result.countries.push(current_country);
            current_country = Country {
                name: stage.country_name.to_owned(),
                competitions: vec![],
            };
        }
        for game in stage.games {
            let status: GameStatus = GameStatus::parse_from_livescore(&game.time)
                .expect("Game status should always parse");
            match chrono::Utc.datetime_from_str(&game.start_time.to_string(), "%Y%m%d%H%M%S") {
                Ok(datetime) => {
                    // There are situations (aka it happened once) where the home or the away team is
                    // empty.
                    let home_team = if !game.home.is_empty() {
                        game.home[0].name.to_owned()
                    } else {
                        String::from("No home team")
                    };
                    let away_team = if !game.away.is_empty() {
                        game.away[0].name.to_owned()
                    } else {
                        String::from("No away team")
                    };
                    let newgame = Game {
                        home_team,
                        away_team,
                        home_score: game.home_score.and_then(|s| s.parse().ok()),
                        away_score: game.away_score.and_then(|s| s.parse().ok()),
                        status,
                        start_time: datetime,
                    };
                    current_competition.games.push(newgame);
                }
                Err(e) => {
                    eprintln!("Failed to parse game start time. Error: {}", e);
                    eprintln!("Skipping this game: {:?}", game);
                }
            }
        }
    }
    current_country.competitions.push(current_competition);
    result.countries.push(current_country);
    result
}

async fn fetch_livescore() -> Result<LiveScore, Box<dyn std::error::Error>> {
    let utc: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
    let oneday = chrono::Duration::days(1);
    let today = format!(
        "https://prod-public-api.livescore.com/v1/api/app/date/soccer/{}/0.00",
        utc.format("%Y%m%d").to_string()
    );
    let yday = format!(
        "https://prod-public-api.livescore.com/v1/api/app/date/soccer/{}/0.00",
        (utc - oneday).format("%Y%m%d").to_string()
    );
    let tomorrow = format!(
        "https://prod-public-api.livescore.com/v1/api/app/date/soccer/{}/0.00",
        (utc + oneday).format("%Y%m%d").to_string()
    );
    let urls = vec![today, yday, tomorrow];
    let mut livescore = LiveScore { stages: vec![] };
    for url in &urls {
        livescore.union(fetch_page(url).await?);
    }
    Ok(livescore)
}

async fn fetch_page(url: &str) -> Result<LiveScore, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let builder = client.get(url).header(
        reqwest::header::USER_AGENT,
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:82.0) Gecko/20100101 Firefox/82.0",
    );
    let result: String = builder.send().await?.text().await?;
    let parsed = serde_json::from_str(&result)?;
    Ok(parsed)
}

#[derive(Serialize, Deserialize, Debug)]
struct LiveScore {
    #[serde(rename = "Stages")]
    stages: Vec<LiveScoreStage>,
}
impl LiveScore {
    /// Sorts countries alphabetically, and the competitions within each country too
    fn _sort_by_alpha(&mut self) {
        self.stages.sort_by(|a, b| {
            a.country_name
                .cmp(&b.country_name)
                .then(a.competition_name.cmp(&b.competition_name))
        });
    }

    /// Sorts by priority. Downside of the everything by country data in Football is that EPL >
    /// Serie A puts *all* competitions in England ahead of those in Italy.
    fn sort_by_priority(&mut self) {
        self.stages.sort();
    }

    fn union(&mut self, mut other: LiveScore) {
        self.stages.append(&mut other.stages);
    }
}
/// Our order is by a priority of importance. This const is used in the implementation of Ord
const COUNTRY_PRIORITIES: &'static [(&'static str, &'static str)] = &[
    ("World Cup", ""),
    ("Euro 2020", ""),
    ("Copa America", ""),
    // Seems to have been renamed to EURO?
    // ("UEFA Nations League", ""),
    ("EURO", ""),
    ("Champions League", ""),
    ("Europa League", ""),
    ("Europa Conference League", ""),
    ("England", "Premier League"),
    ("Germany", "Bundesliga"),
    ("Spain", "LaLiga Santander"),
    ("Italy", "Serie A"),
    ("Belgium", "First Division A"),
    ("Belgium", "Cup"),
    ("France", "Ligue 1"),
    ("England", "Sky Bet Championship"),
    ("Belgium", "First Division B"),
];

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
impl std::cmp::PartialEq for LiveScoreStage {
    fn eq(&self, other: &Self) -> bool {
        self.country_name == other.country_name && self.competition_name == other.competition_name
    }
}
impl std::cmp::Eq for LiveScoreStage {}
impl std::cmp::PartialOrd for LiveScoreStage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
/// Our order is by a priority of importance
impl std::cmp::Ord for LiveScoreStage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a_priority = COUNTRY_PRIORITIES
            .iter()
            .position(|(country, _)| *country == self.country_name)
            .unwrap_or(100);
        let b_priority = COUNTRY_PRIORITIES
            .iter()
            .position(|(country, _)| *country == other.country_name)
            .unwrap_or(100);
        match a_priority.cmp(&b_priority) {
            std::cmp::Ordering::Equal => {
                // TODO This incorrectly gives higher priority to Ethiopia because its league is
                // called "Premier League" too. It should just be considered no priority and kept
                // in the alphabetical country order. If I stay explicit in the priority listing it
                // might not matter anyway though.
                let a_priority = COUNTRY_PRIORITIES
                    .iter()
                    .position(|(_, competition)| *competition == self.competition_name)
                    .unwrap_or(100);
                let b_priority = COUNTRY_PRIORITIES
                    .iter()
                    .position(|(_, competition)| *competition == other.competition_name)
                    .unwrap_or(100);
                match a_priority.cmp(&b_priority) {
                    std::cmp::Ordering::Equal => {
                        // If it was found Equal because same country/competition, then comparing
                        // will do nothing (as you want). If it was found Equal because we do not
                        // have it in our list of priorities and the default value was used, then
                        // this comparison will sort things alphabetically.
                        self.country_name
                            .cmp(&other.country_name)
                            .then(self.competition_name.cmp(&other.competition_name))
                    }
                    res => res,
                }
            }
            res => res,
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct LiveScoreGames {
    #[serde(rename = "Eps", default)]
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
    #[serde(rename = "Nm", default)]
    name: String,
}

impl GameStatus {
    fn parse_from_livescore(s: &str) -> Result<Self, ParseGameStatusError> {
        match s {
            // TODO Can we use start_time immediately?
            "NS" => Ok(GameStatus::Upcoming),
            // TODO AP (After Penalties) puts a * at the winner, but this is not parsed. So you
            // dont know who actually won the game in that case.
            // TODO Also now the AET and AP information is lost completely. Add another variant to
            // the enum?
            "FT" | "AET" | "AP" => Ok(GameStatus::Ended),
            "Postp." => Ok(GameStatus::Postponed),
            "Canc." | "Aband." => Ok(GameStatus::Cancelled),
            // TODO: Only want this for in game time indications (Minutes + HT + ???)
            t => Ok(GameStatus::Ongoing(t.to_owned())),
            // _ => Err(ParseClubLeaderboardSortError),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ParseGameStatusError;
impl std::error::Error for ParseGameStatusError {
    fn description(&self) -> &str {
        "Failed to parse game status."
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
impl std::fmt::Display for ParseGameStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to parse game status.")
    }
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

    #[test]
    fn bad_livescore_parsing() {
        // Encountered a bug when France Cup was parsed right after Belgium Cup. Games from Belgium
        // Cup ended up in France Cup's list. Failure in the parsing logic.
        let livescore = LiveScore {
            stages: vec![
                LiveScoreStage {
                    country_name: String::from("Belgium"),
                    competition_name: String::from("Cup"),
                    games: vec![LiveScoreGames {
                        time: String::from("NS"),
                        start_time: 20210210160000, // Nasty, see the parsing side
                        home: vec![LiveScoreTeam {
                            name: String::from("A"),
                        }],
                        away: vec![LiveScoreTeam {
                            name: String::from("B"),
                        }],
                        home_score: None,
                        away_score: None,
                    }],
                },
                LiveScoreStage {
                    country_name: String::from("France"),
                    competition_name: String::from("Cup"),
                    games: vec![LiveScoreGames {
                        time: String::from("NS"),
                        start_time: 20210210160000, // Nasty, see the parsing side
                        home: vec![LiveScoreTeam {
                            name: String::from("C"),
                        }],
                        away: vec![LiveScoreTeam {
                            name: String::from("D"),
                        }],
                        home_score: None,
                        away_score: None,
                    }],
                },
            ],
        };
        let football = parse_livescore(livescore);
        println!("{:#?}", football);
        assert_eq!(football.countries.len(), 2);
        assert_eq!(football.countries[0].competitions.len(), 1);
        assert_eq!(football.countries[1].competitions.len(), 1);
        assert_eq!(football.countries[0].competitions[0].games.len(), 1);
        assert_eq!(football.countries[1].competitions[0].games.len(), 1);
    }
}
