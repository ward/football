//! <https://www.bbc.com/sport/football/belgian-pro-league/table>

mod search;
mod table;

pub use search::Search;
use std::collections::HashMap;
use table::League;

#[derive(Debug)]
pub struct Beebs {
    /// URL -> [League]
    leagues: HashMap<String, Vec<CachedLeague>>,
    search: search::Search,
}

impl Beebs {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let url = "/sport/football/belgian-pro-league/table";
        let full_url = format!("https://www.bbc.com{}", url);
        let belgian_table = fetch_page(&full_url).await?;
        let mut search = Search::new();
        search.update_data(&belgian_table)?;
        log::trace!("{:#?}", search);
        let leagues = League::from(&belgian_table);
        let cached_leagues = leagues
            .into_iter()
            .map(|league| CachedLeague::new(league))
            .collect();
        let mut leagues = HashMap::new();
        leagues.insert(url.to_string(), cached_leagues);
        Ok(Self { leagues, search })
    }

    /// Get first result of first league. Not quite the approach I want, I think? Idk, it is a lot
    /// of many to even more relations which will be hella messy in the final IRC interface.
    pub fn get_league(&self, query: &str) -> Option<&League> {
        let results = self.search.search(query);
        if results.is_empty() {
            return None;
        }
        let (_score, _key, values) = results.get(0).unwrap();
        if values.is_empty() {
            return None;
        }
        let url = values.get(0)?;
        let leagues = self.leagues.get(url)?;
        let league = leagues.get(0)?;
        Some(&league.league)
    }
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}

#[derive(Debug)]
struct CachedLeague {
    league: League,
    last_updated: std::time::Instant,
}

impl CachedLeague {
    fn new(league: League) -> Self {
        CachedLeague {
            league,
            last_updated: std::time::Instant::now(),
        }
    }
}
