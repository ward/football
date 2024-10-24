use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::Deserialize;

#[derive(Debug)]
pub struct League {
    name: String,
    pub entries: Vec<Entry>,
}

impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "League: {}", self.name)
    }
}

#[derive(Debug)]
pub struct Entry {
    rank: i8,
    team: String,
    win: i8,
    draw: i8,
    lose: i8,
    gf: i8,
    ga: i8,
    points: i8,
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{rank}. {team} {points}pts {win}-{draw}-{lose} {gf}-{ga}",
            rank = self.rank,
            team = self.team,
            points = self.points,
            win = self.win,
            draw = self.draw,
            lose = self.lose,
            gf = self.gf,
            ga = self.ga
        )
    }
}

impl League {
    /// Parses all leagues and groups present in the given content. Empty vec in case of failure.
    pub fn from(content: &str) -> Vec<Self> {
        if let Some(json_blob) = Self::find_json_blob(content) {
            match serde_json::from_str(&json_blob) {
                Ok::<BeebsInitialData, _>(initial_data) => initial_data.gather_leagues(),
                Err(e) => {
                    eprintln!("Failed to parse: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    fn find_json_blob(content: &str) -> Option<String> {
        let data_start = content.find("__INITIAL_DATA__")?;
        let data_end = content[data_start..].find("</script>")? + data_start;
        // start: skip past needle and string open
        // end: -2 because for some reason the json blob is in a string so has "; at the end
        let result = content[data_start + 18..data_end - 2].replace("\\", "");
        // let meta_position = content[needle_position..].find("{\"meta\":")? + needle_position;
        // Not -1 because the range already excludes this position
        // let end_position = content[meta_position..].find(");")? + meta_position;
        Some(result)
    }

    /// Gets all ranked entries
    pub fn get_ranking(&self) -> &Vec<Entry> {
        &self.entries
    }

    /// Gets up to 6 teams around a certain position
    pub fn get_ranking_around(&self, idx: usize) -> &[Entry] {
        let length = self.entries.len();
        let range = if length <= 6 {
            0..length
        } else if idx <= 3 {
            0..6
        } else if idx >= length - 2 {
            (length - 6)..length
        } else {
            (idx - 3)..(idx + 3)
        };
        &self.entries[range]
    }

    /// Returns 0 indexed position.
    /// Defaults to 0 if nothing found.
    /// Yes that makes little sense but we're only using this in one place.
    pub fn find_team_position(&self, needle: &str) -> usize {
        let needle = needle.to_lowercase();
        for rank in &self.entries {
            let team_name = rank.team.to_lowercase();
            if team_name.matches(&needle).count() > 0 {
                return (rank.rank - 1).try_into().unwrap_or(0);
            }
        }
        0
    }
}

#[derive(Deserialize, Debug)]
struct BeebsInitialData {
    data: BeebsInnerData,
}
impl BeebsInitialData {
    fn gather_leagues(&self) -> Vec<League> {
        let mut result = vec![];
        for tournament in &self.data.football_table.data.tournaments {
            for stage in &tournament.stages {
                for round in &stage.rounds {
                    let name = if stage.name.eq_ignore_ascii_case("Regular Season") {
                        tournament.name.clone()
                    } else if let Some(roundname) = &round.name {
                        format!("{} {} {}", tournament.name, stage.name, roundname)
                    } else {
                        format!("{} {}", tournament.name, stage.name)
                    };
                    let mut league = League {
                        name,
                        entries: vec![],
                    };
                    for participant in &round.participants {
                        league.entries.push(participant.to_entry());
                    }
                    result.push(league);
                }
            }
        }
        result
    }
}

#[derive(Debug)]
struct BeebsInnerData {
    football_table: BeebsFootballTable,
}
// Need to do some funky stuff here because the football-table property comes with
// `?bunchofgarbage` as well. Did not see something built-in to handle that so doing it this way
impl<'de> Deserialize<'de> for BeebsInnerData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BeebsInnerDataVisitor;

        impl<'de> Visitor<'de> for BeebsInnerDataVisitor {
            type Value = BeebsInnerData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("`football-table` with optional mumbo jumbo after")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut football_table = None;

                while let Some((key, value)) = access.next_entry::<String, serde_json::Value>()? {
                    if key.starts_with("football-table") {
                        // Parse the value as TableData
                        football_table =
                            Some(serde_json::from_value(value).map_err(de::Error::custom)?);
                    }
                    // Ignore other fields
                }

                let football_table = football_table
                    .ok_or_else(|| de::Error::custom("missing football-table field"))?;

                Ok(BeebsInnerData { football_table })
            }
        }

        deserializer.deserialize_map(BeebsInnerDataVisitor)
    }
}

#[derive(Deserialize, Debug)]
struct BeebsFootballTable {
    data: BeebsFootballTableData,
}
#[derive(Deserialize, Debug)]
struct BeebsFootballTableData {
    tournaments: Vec<BeebsTournament>,
}
#[derive(Deserialize, Debug)]
struct BeebsTournament {
    name: String,
    stages: Vec<BeebsStages>,
}
#[derive(Deserialize, Debug)]
struct BeebsStages {
    name: String,
    rounds: Vec<BeebsRounds>,
}
#[derive(Deserialize, Debug)]
struct BeebsRounds {
    name: Option<String>,
    participants: Vec<BeebsParticipant>,
}
#[derive(Deserialize, Debug)]
struct BeebsParticipant {
    rank: i8,
    name: String,
    points: i8,
    wins: i8,
    losses: i8,
    draws: i8,
    #[serde(rename = "goalsScoredFor")]
    gf: i8,
    #[serde(rename = "goalsScoredAgainst")]
    ga: i8,
}
impl BeebsParticipant {
    fn to_entry(&self) -> Entry {
        Entry {
            rank: self.rank,
            team: self.name.clone(),
            win: self.wins,
            draw: self.draws,
            lose: self.losses,
            gf: self.gf,
            ga: self.ga,
            points: self.points,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_belgium_table() {
        let _ = env_logger::builder().is_test(true).try_init();

        let content = include_str!("belgium.1a.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 1);
    }

    #[test]
    fn parse_cl() {
        let _ = env_logger::builder().is_test(true).try_init();
        let content = include_str!("cl.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 1);

        let combinedtable = leagues.get(0).unwrap();
        assert_eq!(combinedtable.name, "UEFA Champions League League Stage");

        assert_eq!(combinedtable.entries.get(0).unwrap().team, "Aston Villa");
    }

    #[test]
    fn parse_nations_league() {
        let _ = env_logger::builder().is_test(true).try_init();
        let content = include_str!("nations_league.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 14);

        let group_f = leagues.get(5).unwrap();
        assert_eq!(group_f.name, "UEFA Nations League League C Group 2");

        assert_eq!(group_f.entries.get(0).unwrap().team, "Romania");
    }

    #[test]
    fn parse_premier_league() {
        let _ = env_logger::builder().is_test(true).try_init();
        let content = include_str!("epl.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 1);
        let epl = leagues.get(0).unwrap();
        assert_eq!(epl.entries.get(2).unwrap().team, "Arsenal");
    }
}
