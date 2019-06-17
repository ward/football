use std::error;
use std::fmt;
use std::str::FromStr;

mod livescore;

pub fn get_all_games() -> Result<Football, Box<dyn std::error::Error>> {
    livescore::get_all_games()
}

#[derive(Debug, Clone)]
pub struct Football {
    pub countries: Vec<Country>,
}
impl Football {
    // TODO This would be much nicer with on-line approximate string matching.
    //      bitap algorithm or somesuch?
    /// Splits string into pieces, only keeps games for which every piece is matched by either
    /// country, competition, or teams
    pub fn query(self, query: &str) -> Football {
        let query: Vec<_> = query
            .split(|c: char| !c.is_ascii_alphabetic())
            .map(|word| word.to_lowercase())
            .collect();
        let mut games = Football { countries: vec![] };
        for country in &self.countries {
            let mut filteredcompetitions = vec![];
            for competition in &country.competitions {
                let filteredgames: Vec<_> = competition
                    .games
                    .iter()
                    .filter(|game| {
                        query.iter().all(|word| {
                            country.name.to_lowercase().contains(word)
                                || competition.name.to_lowercase().contains(word)
                                || game.home_team.to_lowercase().contains(word)
                                || game.away_team.to_lowercase().contains(word)
                        })
                    })
                    .cloned()
                    .collect();
                if !filteredgames.is_empty() {
                    filteredcompetitions.push(Competition {
                        name: competition.name.to_owned(),
                        games: filteredgames,
                    });
                }
            }
            if !filteredcompetitions.is_empty() {
                games.countries.push(Country {
                    name: country.name.to_owned(),
                    competitions: filteredcompetitions,
                });
            }
        }
        games
    }

    pub fn number_of_games(&self) -> usize {
        let mut ctr = 0;
        for country in &self.countries {
            for competition in &country.competitions {
                ctr += competition.games.len();
            }
        }
        ctr
    }

    // TODO: Time querying (now, today, tomorrow, ended, ...)
}
#[derive(Debug, Clone)]
pub struct Country {
    pub name: String,
    pub competitions: Vec<Competition>,
}
#[derive(Debug, Clone)]
pub struct Competition {
    pub name: String,
    pub games: Vec<Game>,
}
#[derive(Debug, Clone)]
pub struct Game {
    home_team: String,
    away_team: String,
    home_score: Option<u8>,
    away_score: Option<u8>,
    status: GameStatus,
}
impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.status {
            GameStatus::Ended => write!(
                f,
                "(FT) {home} {home_score}-{away_score} {away}",
                home = self.home_team,
                home_score = self.home_score.unwrap_or(100),
                away_score = self.away_score.unwrap_or(100),
                away = self.away_team
            ),
            GameStatus::Upcoming(t) => write!(f, "({}) {} - {}", t, self.home_team, self.away_team),
            GameStatus::Ongoing(t) => write!(
                f,
                "({}) {} {}-{} {}",
                t,
                self.home_team,
                self.home_score.unwrap_or(100),
                self.away_score.unwrap_or(100),
                self.away_team
            ),
            GameStatus::Postponed => {
                write!(f, "(postponed) {} - {}", self.home_team, self.away_team)
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
enum GameStatus {
    Upcoming(u64), // TODO: Replace by Chrono
    Ongoing(String),
    Ended,
    Postponed,
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
            "Postp." => Ok(GameStatus::Postponed),
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
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
impl fmt::Display for ParseGameStatusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Failed to parse game status.")
    }
}
