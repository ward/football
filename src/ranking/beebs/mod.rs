//! https://www.bbc.com/sport/football/belgian-pro-league/table

use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

pub struct Beebs {
    pub search: Search,
}

impl Beebs {
    pub async fn new() -> Result<(), Box<dyn std::error::Error>> {
        let belgian_table =
            fetch_page("https://www.bbc.com/sport/football/belgian-pro-league/table").await?;
        let tables_info = Self::get_tables_info(&belgian_table)?;
        log::trace!("{}", tables_info);
        let mut search = Search::new();
        search.update_data(tables_info);
        log::trace!("{:#?}", search);
        Ok(())
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
}

#[derive(Debug)]
enum SearchError {
    DidNotFindRawTableInfo,
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SearchError::DidNotFindRawTableInfo => {
                write!(f, "Could not find the text for table searching.")
            }
        }
    }
}

impl std::error::Error for SearchError {
    fn description(&self) -> &str {
        match self {
            SearchError::DidNotFindRawTableInfo => "Raw text for table searching not found",
        }
    }
}

impl fmt::Debug for Search {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Search object. Data {:#?}", self.data)
    }
}

/// Searching perhaps with skim. Here is an example of someone using it non interactively.
/// https://github.com/mrjones2014/caniuse-rs/commit/a6099dcc246c68b079f21b33b5c8f2ecc8a60d4b
/// Perhaps better with https://crates.io/crates/fuzzy-matcher, though then it is only fuzzy, not
/// easy switching to regex and stuff I think.
pub struct Search {
    hashed: String,
    /// From a potential search hit (e.g., Belgium) to a possible URL (e.g., the one for the 1st
    /// division in Belgium).
    /// TODO: One hit can point to many urls so need to rethink the value part of the hashmap
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

    /// Updates the data if the input is different from input parsed before.
    pub fn update_data(&mut self, input: &str) {
        if self.hashed != Self::hash_input(input) {
            self.force_update_data(input);
        }
    }

    fn force_update_data(&mut self, input: &str) {
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
            }
            // TODO Make this return a result
            Err(e) => log::error!("{}", e),
        }
    }

    pub fn search(&self, needle: &str) -> Vec<(i64, &String)> {
        // TODO Decide on the exact interface for this
        let mut matches = vec![];
        for (key, _values) in &self.data {
            if let Some(score) = self.fuzzy_matcher.fuzzy_match(key, needle) {
                matches.push((score, key));
            }
        }
        matches.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());
        matches
    }
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
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
        env_logger::init();
        let content = include_str!("belgium.1a.html");
        let tables_info = Beebs::get_tables_info(&content).unwrap();
        let mut search = Search::new();
        search.update_data(tables_info);
        assert_eq!(
            vec![(149, &"rsc anderlecht".to_string())],
            search.search("ANDelech")
        );
    }
}
