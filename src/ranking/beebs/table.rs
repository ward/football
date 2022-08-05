use serde::Deserialize;

#[derive(Debug)]
pub struct League {
    name: String,
    url: String,
    pub entries: Vec<Entry>,
}

impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "League: {} at {}", self.name, self.url)
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
    pub fn from(content: &str) -> Vec<Self> {
        if let Some(json_blob) = Self::find_json_blob(content) {
            match serde_json::from_str(json_blob) {
                Ok::<ParseRanks, _>(parsed) => parsed.to_leagues(),
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
                format!(
                    "{} {}",
                    competition_name,
                    group_name
                )
            } else {
                competition_name.to_string()
            };
            let url = String::from("");
            let entries: Vec<Entry> = self
                .body
                .sport_tables
                .tables
                .get(0)
                .unwrap()
                .rows
                .iter()
                .map(|row| row.to_entry())
                .collect();
            leagues.push(League { name, url, entries })
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
        assert_eq!(group_f.name, "Champions League Group F");
    }
}
