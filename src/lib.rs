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

    // TODO: I want both "women world cup" and "world cup women" to match the same manner.
    // Idea: split up and see if I can just add the match number for each to get something
    // meaningful? (Perhaps this is more something for the bitap library side)
    pub fn fuzzy_query(self, query: &str) -> Vec<(f64, Country, Competition, Game)> {
        let query = query.to_lowercase();
        let bitap = bitap::Bitap::new().distance(100_000).threshold(0.3);
        let mut result = vec![];
        for country in &self.countries {
            for competition in &country.competitions {
                for game in &competition.games {
                    let fullstr = format!(
                        "{} {} {} {}",
                        country.name, competition.name, game.home_team, game.away_team
                    )
                    .to_lowercase();
                    let matcher = bitap.bitap(&fullstr, &query);
                    if matcher.is_match {
                        result.push((
                            matcher.score,
                            country.clone(),
                            competition.clone(),
                            game.clone(),
                        ));
                    }
                }
            }
        }
        result.sort_by(|(score1, _, _, _), (score2, _, _, _)| score1.partial_cmp(score2).unwrap());
        result
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
            GameStatus::Cancelled => {
                write!(f, "(cancelled) {} - {}", self.home_team, self.away_team)
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
    Cancelled,
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

// TODO The following should probably be livescore specific.
impl FromStr for GameStatus {
    type Err = ParseGameStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // TODO Can we use start_time immediately?
            "NS" => Ok(GameStatus::Upcoming(0)),
            "FT" | "AET" => Ok(GameStatus::Ended),
            "Postp." => Ok(GameStatus::Postponed),
            "Canc." => Ok(GameStatus::Cancelled),
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
