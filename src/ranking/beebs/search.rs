use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

/// Uses <https://crates.io/crates/fuzzy-matcher> under the hood
pub struct Search {
    hashed: String,
    /// From a potential search hit (e.g., Belgium) to a possible URL (e.g., the one for the 1st
    /// division in Belgium).
    data: HashMap<String, Vec<String>>,
    fuzzy_matcher: fuzzy_matcher::skim::SkimMatcherV2,
}

impl Search {
    pub fn new() -> Self {
        // This has an (enabled by default) cache thing, what does it do exactly? Is it useful for
        // our reusing of the SkimMatcherV2? TODO
        let fuzzy_matcher = fuzzy_matcher::skim::SkimMatcherV2::default().ignore_case();
        let hashed = String::from("");
        let data = HashMap::new();
        Self {
            hashed,
            data,
            fuzzy_matcher,
        }
    }

    fn hash_input(input: &str) -> String {
        input.to_string()
    }

    /// Updates the data if the search part is different from search parsed before.
    pub fn update_data(&mut self, input: &str) -> Result<(), SearchError> {
        let table_info_json = Self::get_tables_info(input)?;
        if self.hashed != Self::hash_input(table_info_json) {
            self.force_update_data(table_info_json)
        } else {
            Ok(())
        }
    }

    /// Takes the json part of the page that represents the info we use for searching.
    fn force_update_data(&mut self, input: &str) -> Result<(), SearchError> {
        self.hashed = Self::hash_input(input);
        match serde_json::from_str(input) {
            Ok::<ParseSearch, _>(parsed) => {
                log::trace!("Parsed: {:#?}", parsed);
                let leagues = parsed.body;
                for league in leagues {
                    // These seem to be team pages we don't care for
                    if league.url.starts_with("/sport/football/teams/") {
                        continue;
                    }
                    let mut keys = league.alternatives;
                    keys.push(league.name);
                    for key in keys {
                        match self.data.get_mut(&key) {
                            Some(value) => value.push(league.url.to_string()),
                            None => {
                                self.data.insert(key, vec![league.url.to_string()]);
                            }
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(SearchError::CouldNotParseSearch(e.to_string())),
        }
    }

    fn get_tables_info(content: &str) -> Result<&str, SearchError> {
        // The relevant line contains this needle
        let needle_position = content
            .find("bbc-morph-sport-teams-competitions-list")
            .ok_or(SearchError::DidNotFindRawTableInfo)?;
        let meta_position = content[needle_position..]
            .find("{\"meta\":")
            .ok_or(SearchError::DidNotFindRawTableInfo)?
            + needle_position;
        // Not -1 because the range already excludes this position
        let end_position = content[meta_position..]
            .find(");")
            .ok_or(SearchError::DidNotFindRawTableInfo)?
            + meta_position;
        Ok(&content[meta_position..end_position])
    }

    pub fn search(&self, needle: &str) -> Vec<(i64, &String, &Vec<String>)> {
        // TODO Decide on the exact interface for this
        let mut matches = vec![];
        // TODO Add values in the result since that will be the connection to more
        for (key, values) in &self.data {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(key, needle) {
                matches.push((score, key, values));
            }
        }
        matches.sort_unstable_by(|(a_score, a_name, _a_url), (b_score, b_name, _b_url)| {
            b_score.cmp(a_score).then_with(|| a_name.cmp(b_name))
        });
        matches
    }
}

#[derive(Debug)]
pub enum SearchError {
    DidNotFindRawTableInfo,
    CouldNotParseSearch(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SearchError::DidNotFindRawTableInfo => {
                write!(f, "Could not find the text for table searching.")
            }
            SearchError::CouldNotParseSearch(_) => {
                write!(f, "Could not parse search info")
            }
        }
    }
}

impl std::error::Error for SearchError {
    fn description(&self) -> &str {
        match self {
            SearchError::DidNotFindRawTableInfo => "Raw text for table searching not found",
            SearchError::CouldNotParseSearch(_) => "Could not parse search info",
        }
    }
}

impl fmt::Debug for Search {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Search object. Data {:#?}", self.data)
    }
}

#[derive(Deserialize, Debug)]
struct ParseSearch {
    body: Vec<ParseLeague>,
}

#[derive(Deserialize, Debug)]
struct ParseLeague {
    name: String,
    url: String,
    #[serde(default)]
    alternatives: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_search() {
        let _ = env_logger::builder().is_test(true).try_init();

        let content = include_str!("belgium.1a.html");
        let mut search = Search::new();
        search.update_data(&content).unwrap();
        assert_eq!(
            vec![(149, &"rsc anderlecht".to_string(), &vec!["/sport/football/belgian-pro-league/table".to_string()])],
            search.search("ANDelech")
        );
    }
}
