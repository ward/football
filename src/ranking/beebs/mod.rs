//! <https://www.bbc.com/sport/football/belgian-pro-league/table>

mod search;
mod table;

pub use search::Search;
use table::League;

pub struct Beebs {
    // TODO There needs to be some order here, name or url to league or whatever
    pub leagues: Vec<League>,
    pub search: search::Search,
}

impl Beebs {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let belgian_table =
            fetch_page("https://www.bbc.com/sport/football/belgian-pro-league/table").await?;
        let mut search = Search::new();
        search.update_data(&belgian_table)?;
        log::trace!("{:#?}", search);
        let leagues = League::from(&belgian_table);
        Ok(Self { leagues, search })
    }
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}
