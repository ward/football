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
            match serde_json::from_str(json_blob) {
                Ok::<ParseRanks, _>(parsed) => parsed.to_leagues(),
                Err(e) => {
                    eprintln!("Failed to parse: {}", e);
                    vec![]
                }
            }
        } else if let Some(json_blob) = Self::find_epl_json_blob(content) {
            // Very rudimentary cleanup
            let json_blob = json_blob.replace("\\\"", "\"");
            log::trace!("cleaned epl_json_blob: {}", json_blob);
            match serde_json::from_str(&json_blob) {
                Ok::<ParseEplRanks, _>(parsed) => {
                    log::trace!("Parsed epl json: {:#?}", parsed);
                    parsed.to_leagues()
                }
                Err(e) => {
                    eprintln!("Failed to parse: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        }
    }

    fn find_json_blob(content: &str) -> Option<&str> {
        let needle_position = content.find("bbc-morph-sport-tables-data")?;
        let meta_position = content[needle_position..].find("{\"meta\":")? + needle_position;
        // Not -1 because the range already excludes this position
        let end_position = content[meta_position..].find(");")? + meta_position;
        Some(&content[meta_position..end_position])
    }

    /// They got a special table for the EPL which also decides to be completely differently
    /// implemented...
    fn find_epl_json_blob(content: &str) -> Option<&str> {
        let needle_position = content.find("{\\\"participants\\\"")?;
        log::trace!("needle_position: {}", needle_position);
        let end_position = content[needle_position..].find("}]}]}]")? + 3 + needle_position;
        log::trace!("end_position: {}", end_position);
        Some(&content[needle_position..end_position])
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
struct ParseRanks {
    body: ParseSportTables,
}
#[derive(Deserialize, Debug)]
struct ParseSportTables {
    #[serde(rename = "sportTables")]
    sport_tables: ParseSportTable,
}
#[derive(Deserialize, Debug)]
struct ParseSportTable {
    title: String,
    tables: Vec<ParseActualTable>,
}
#[derive(Deserialize, Debug)]
struct ParseActualTable {
    group: ParseGroupMeta,
    rows: Vec<ParseRow>,
}
#[derive(Deserialize, Debug)]
struct ParseGroupMeta {
    name: Option<String>,
}
#[derive(Deserialize, Debug)]
struct ParseRow {
    cells: Vec<ParseCell>,
}
#[derive(Deserialize, Debug)]
struct ParseCell {
    td: ParseTd,
}
#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum ParseTd {
    ParseTdString {
        text: String,
    },
    ParseTdNumber {
        text: i8,
    },
    ParseTdForm {
        form: Vec<ParseForm>,
    },
    ParseTdLink {
        #[serde(rename = "abbrLink")]
        abbr_link: ParseLink,
    },
}
#[derive(Deserialize, Debug)]
struct ParseForm {
    result: String,
}
#[derive(Deserialize, Debug)]
struct ParseLink {
    text: String,
}

impl ParseRanks {
    fn to_leagues(&self) -> Vec<League> {
        let competition_name = if self.body.sport_tables.title.ends_with(" Tables") {
            // TODO Cut off the last part or use a replace or somesuch. Maybe go to chars and back?
            self.body.sport_tables.title.to_string()
        } else {
            self.body.sport_tables.title.to_string()
        };
        let mut leagues = vec![];
        for league in &self.body.sport_tables.tables {
            let name = if let Some(group_name) = &league.group.name {
                format!("{} {}", competition_name, group_name)
            } else {
                competition_name.to_string()
            };
            let entries: Vec<Entry> = league.rows.iter().map(|row| row.to_entry()).collect();
            leagues.push(League { name, entries })
        }
        leagues
    }
}

impl ParseRow {
    fn to_entry(&self) -> Entry {
        let rank = self.cells.get(0).unwrap().td.to_inner_number();
        // position 1 is whether team moved up or down
        let team = self.cells.get(2).unwrap().td.to_inner_string();
        // position 3 is played
        let win = self.cells.get(4).unwrap().td.to_inner_number();
        let draw = self.cells.get(5).unwrap().td.to_inner_number();
        let lose = self.cells.get(6).unwrap().td.to_inner_number();
        let gf = self.cells.get(7).unwrap().td.to_inner_number();
        let ga = self.cells.get(8).unwrap().td.to_inner_number();
        // gd is position 9
        let points = self.cells.get(10).unwrap().td.to_inner_number();

        Entry {
            rank,
            team,
            win,
            draw,
            lose,
            gf,
            ga,
            points,
        }
    }
}
impl ParseTd {
    fn to_inner_string(&self) -> String {
        match self {
            ParseTd::ParseTdString { text } => text.to_string(),
            ParseTd::ParseTdForm { form } => form
                .iter()
                .map(|pf| pf.result.to_string())
                .collect::<Vec<_>>()
                .join(""),
            ParseTd::ParseTdNumber { text } => text.to_string(),
            ParseTd::ParseTdLink { abbr_link } => abbr_link.text.to_string(),
        }
    }
    fn to_inner_number(&self) -> i8 {
        match self {
            ParseTd::ParseTdNumber { text } => *text,
            _ => panic!("Fix me"),
        }
    }
}

#[derive(Deserialize, Debug)]
struct ParseEplRanks {
    participants: Vec<ParseEplParticipant>,
}
#[derive(Deserialize, Debug)]
struct ParseEplParticipant {
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
impl ParseEplRanks {
    fn to_leagues(self) -> Vec<League> {
        let entries = self
            .participants
            .into_iter()
            .map(|part| part.to_entry())
            .collect();
        vec![League {
            name: "English Premier League".to_string(),
            entries,
        }]
    }
}
impl ParseEplParticipant {
    fn to_entry(self) -> Entry {
        Entry {
            rank: self.rank,
            team: self.name,
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
    fn parse_table() {
        let _ = env_logger::builder().is_test(true).try_init();

        let content = include_str!("belgium.1a.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 1);
    }

    #[test]
    fn parse_groups() {
        let _ = env_logger::builder().is_test(true).try_init();
        let content = include_str!("cl.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 8);

        let group_f = leagues.get(5).unwrap();
        assert_eq!(group_f.name, "Champions League Tables Group F");

        assert_eq!(group_f.entries.get(0).unwrap().team, "Manchester United");
    }

    #[test]
    fn parse_premier_league() {
        let _ = env_logger::builder().is_test(true).try_init();
        let content = include_str!("epl.html");

        let leagues = League::from(content);
        assert_eq!(leagues.len(), 1);
        let epl = leagues.get(0).unwrap();
        assert_eq!(epl.entries.get(2).unwrap().team, "Liverpool");
    }
}
