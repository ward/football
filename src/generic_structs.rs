use std::fmt;

/// Container struct for all football scores
#[derive(Debug, Clone)]
pub struct Football {
    pub countries: Vec<Country>,
}

/// Representing a country or major competition (CL, EL, WC, ...)
#[derive(Debug, Clone)]
pub struct Country {
    pub name: String,
    pub competitions: Vec<Competition>,
}

/// Representing a league, or a group or stage in a major competition
#[derive(Debug, Clone)]
pub struct Competition {
    pub name: String,
    pub games: Vec<Game>,
}

/// One game of football, possibly future, past, or present
#[derive(Debug, Clone)]
pub struct Game {
    pub home_team: String,
    pub away_team: String,
    pub home_score: Option<u8>,
    pub away_score: Option<u8>,
    pub status: GameStatus,
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
pub enum GameStatus {
    Upcoming(u64), // TODO: Replace by Chrono (otoh extra dep)
    Ongoing(String),
    Ended,
    Postponed,
    Cancelled,
    // Other(String),
}

impl GameStatus {
    pub fn set_start_time(self, start_time: u64) -> GameStatus {
        if let GameStatus::Upcoming(_) = self {
            GameStatus::Upcoming(start_time)
        } else {
            self
        }
    }
}
