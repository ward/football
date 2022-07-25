//! https://www.bbc.com/sport/football/belgian-pro-league/table

mod search;
use search::*;

pub struct Beebs {
    pub search: search::Search,
}

impl Beebs {
    pub async fn new() -> Result<(), Box<dyn std::error::Error>> {
        let belgian_table =
            fetch_page("https://www.bbc.com/sport/football/belgian-pro-league/table").await?;
        let mut search = Search::new();
        search.update_data(&belgian_table)?;
        log::trace!("{:#?}", search);
        Ok(())
    }
}

async fn fetch_page(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?.text().await?;
    Ok(response)
}
