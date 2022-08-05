//! <https://www.bbc.com/sport/football/belgian-pro-league/table>

mod search;
mod table;

pub use search::Search;
use std::collections::HashMap;
use table::League;

#[derive(Debug)]
pub struct Beebs {
    /// URL -> [League]
    leagues: HashMap<String, CachedLeagues>,
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
        let cached_leagues = CachedLeagues::new(&full_url, leagues);
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
        leagues.get(0)
    }
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}

const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(10 * 60);

#[derive(Debug)]
pub struct CachedLeagues {
    leagues: Vec<League>,
    url: String,
    last_updated: std::time::Instant,
}

impl CachedLeagues {
    pub fn new(url: &str, leagues: Vec<League>) -> Self {
        Self {
            leagues,
            url: url.to_string(),
            last_updated: std::time::Instant::now(),
        }
    }

    pub fn empty(url: &str) -> Self {
        Self {
            leagues: vec![],
            url: url.to_string(),
            last_updated: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .unwrap()
                .checked_sub(CACHE_DURATION)
                .unwrap(),
        }
    }

    /// Updates if last update is older than CACHE_DURATION
    pub async fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.needs_update() {
            println!("Fetching from {}", self.url);
            let page = fetch_page(&self.url).await?;
            let leagues = League::from(&page);
            if self.leagues.len() <= leagues.len() {
                println!("Updating");
                println!("{:#?}", leagues);
                self.leagues = leagues;
                self.last_updated = std::time::Instant::now();
            }
        }
        Ok(())
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_updated);
        passed_time > CACHE_DURATION
    }

    pub fn get(&self, idx: usize) -> Option<&League> {
        self.leagues.get(idx)
    }
}
