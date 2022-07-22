//! Seems to be blocking at least some of the IPs that I tend to use soooo, has sort of lost its
//! usefulness to me now. Bummer.

use log::{debug, warn};
use scraper::Html;
use scraper::Selector;
use std::collections::HashMap;

const CACHE_DURATION: std::time::Duration = std::time::Duration::from_secs(10 * 60);

// TODO Should League and Competition (Group) have one Trait as interface? Lots of code repetition atm

#[derive(Debug)]
pub struct League {
    ranking: Vec<RankingEntry>,
    url: String,
    last_updated: std::time::Instant,
}

impl League {
    pub fn new(url: String) -> Self {
        Self {
            url,
            ranking: vec![],
            last_updated: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .unwrap(),
        }
    }

    /// Updates if last update is older than CACHE_DURATION
    pub async fn update(&mut self) -> Result<(), reqwest::Error> {
        if self.needs_update() {
            debug!("Fetching data from {}", self.url);
            self.last_updated = std::time::Instant::now();
            let client = create_client().await?;
            let resp = client
                .get(&self.url)
                .version(reqwest::Version::HTTP_11)
                .send()
                .await?;
            if resp.status().is_success() {
                let content = resp.text().await?;
                self.ranking = parse_ranking(&content);
            } else {
                // TODO return error. Need to make my own error type then that also wraps the
                // reqwest ones.
                warn!("Request was not successful, status: {}", resp.status());
            }

            Ok(())
        } else {
            Ok(())
        }
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_updated);
        passed_time > CACHE_DURATION
    }

    /// Gets up to 6 teams around a certain position
    pub fn get_ranking_around(&self, idx: usize) -> &[RankingEntry] {
        let length = self.ranking.len();
        let range = if length <= 6 {
            0..length
        } else if idx <= 3 {
            0..6
        } else if idx >= length - 2 {
            (length - 6)..length
        } else {
            (idx - 3)..(idx + 3)
        };
        &self.ranking[range]
    }

    /// Returns 0 indexed position.
    /// Defaults to 0 if nothing found.
    /// Yes that makes little sense but we're only using this in one place.
    pub fn find_team_position(&self, needle: &str) -> u8 {
        let needle = needle.to_lowercase();
        for rank in &self.ranking {
            let team_name = rank.team.to_lowercase();
            if team_name.matches(&needle).count() > 0 {
                return rank.rank - 1;
            }
        }
        0
    }
}

/// Extracted so both League and Group can use it. Should this be some trait?
/// Returns empty Vec if an error is encountered
///
/// TODO Proper Result returning?
fn parse_ranking(content: &str) -> Vec<RankingEntry> {
    let doc = Html::parse_document(content);
    match Selector::parse("table.leaguetable.sortable.table.detailed-table tbody tr") {
        Ok(selector) => {
            let mut ranking = vec![];
            for row in doc.select(&selector) {
                match RankingEntry::parse_from_row(row) {
                    Some(entry) => ranking.push(entry),
                    None => {
                        warn!("Failed to parse a ranking row, returning empty vector");
                        return vec![];
                    }
                }
            }
            ranking
        }
        Err(e) => {
            warn!("Failed to parse content. {:?}", e);
            vec![]
        }
    }
}

#[derive(Debug)]
pub struct Competition {
    groups: HashMap<String, Group>,
}

impl Competition {
    pub fn new(group_config: &HashMap<String, String>) -> Self {
        let mut groups = HashMap::new();
        for (group_name, group_url) in group_config {
            groups.insert(group_name.clone(), Group::new(group_url.clone()));
        }
        Self { groups }
    }

    pub fn get_group_mut(&mut self, group_id: &str) -> Option<&mut Group> {
        self.groups.get_mut(group_id)
    }
}

/// When created, only stores the source url. Will not fetch the rankings till asked to do so.
#[derive(Debug)]
pub struct Group {
    ranking: Vec<RankingEntry>,
    url: String,
    last_updated: std::time::Instant,
}

impl Group {
    fn new(url: String) -> Self {
        Self {
            url,
            ranking: vec![],
            last_updated: std::time::Instant::now()
                .checked_sub(CACHE_DURATION)
                .unwrap(),
        }
    }

    /// Updates if last update is older than CACHE_DURATION
    pub async fn update(&mut self) -> Result<(), reqwest::Error> {
        if self.needs_update() {
            debug!("Fetching data from {}", self.url);
            self.last_updated = std::time::Instant::now();
            let client = create_client().await?;
            let resp = client
                .get(&self.url)
                .version(reqwest::Version::HTTP_11)
                .send()
                .await?;
            if resp.status().is_success() {
                let content = resp.text().await?;
                self.ranking = parse_ranking(&content);
            } else {
                // TODO return error. Need to make my own error type then that also wraps the
                // reqwest ones.
                warn!("Request was not successful, status: {}", resp.status());
            }

            Ok(())
        } else {
            Ok(())
        }
    }

    /// True if last update is older than CACHE_DURATION
    fn needs_update(&self) -> bool {
        let now = std::time::Instant::now();
        let passed_time = now.duration_since(self.last_updated);
        passed_time > CACHE_DURATION
    }

    pub fn get_ranking(&self) -> &Vec<RankingEntry> {
        &self.ranking
    }
}

/// Both League and Group need the same kind of client
async fn create_client() -> Result<reqwest::Client, reqwest::Error> {
    // Took a bit of trial and error to get it working, but seems to be good now.
    // ... Nop.
    let client_builder = reqwest::ClientBuilder::new();
    let mut headers = reqwest::header::HeaderMap::new();
    // headers.insert(
    //     reqwest::header::COOKIE,
    //     reqwest::header::HeaderValue::from_static("cookies_enabled=; s_cc=true; s_ppv=-%2C72%2C65%2C1485; s_sq=%5B%5BB%5D%5D; user_cc=BE; cookies_enabled=; euconsent-v2=CPcDG4APcDG4AAKAqAENCXCsAP_AAH_AAAwII4Nd_X__bX9j-_5_aft0eY1P9_r37uQzDhfNs-8F3L_W_LwXw2E7NF36pq4KmR4Eu3LBIQNlHMHUTUmwaokVrzHsak2cpyNKJ7JEknMZO2dYGF9Pn9lDuYKY7_5_9_bx2D-t_9_-39T378Xf3_dp_2_--vCfV599jfn9fV_789KP___9v-_8__________38EbwCTDVuIAuzLHBk0DCKFECMKwkKoFABBQDC0RWADg4KdlYBLrCFgAgFSEYEQIMQUYMAgAEEgCQiICQAsEAiAIgEAAIAEQCEABEwCCwAsDAIAAQDQsQAoABAkIMiAiOUwICoEgoJbKxBKCvY0wgDrPACgURsVAAiSQEUgICQsHAcASAl4skDTFC-QAjBCgFEAAAA.fmgACGgAAAAA; addtl_consent=1~39.4.3.9.6.9.13.6.4.15.9.5.2.7.4.1.7.1.3.2.10.3.5.4.21.4.6.9.7.10.2.9.2.18.7.6.14.5.20.6.5.1.3.1.11.29.4.14.4.5.3.10.6.2.9.6.6.4.5.4.4.29.4.5.3.1.6.2.2.17.1.17.10.9.1.8.6.2.8.3.4.142.4.8.42.15.1.14.3.1.8.10.25.3.7.25.5.18.9.7.41.2.4.18.21.3.4.2.1.6.6.5.2.14.18.7.3.2.2.8.20.8.8.6.3.10.4.20.2.13.4.6.4.11.1.3.22.16.2.6.8.2.4.11.6.5.33.11.8.1.10.28.12.1.3.21.2.7.6.1.9.30.17.4.9.15.8.7.3.6.6.7.2.4.1.7.12.13.22.13.2.12.2.10.5.15.2.4.9.4.5.4.7.13.5.15.4.13.4.14.8.2.15.2.5.5.1.2.2.1.2.14.7.4.8.2.9.10.18.12.13.2.18.1.1.3.1.1.9.25.4.1.19.8.4.5.3.5.4.8.4.2.2.2.14.2.13.4.2.6.9.6.3.4.3.5.2.3.6.10.11.6.3.16.3.11.3.1.2.3.9.19.11.15.3.10.7.6.4.3.4.6.3.3.3.3.1.1.1.6.11.3.1.1.11.6.1.10.5.2.6.3.2.2.4.3.2.2.7.15.7.12.2.1.3.3.4.5.4.3.2.2.4.1.3.1.1.1.2.9.1.6.9.1.5.2.1.7.2.8.11.1.3.1.1.2.1.3.2.6.1.12.5.3.1.3.1.1.2.2.7.7.1.4.1.2.6.1.2.1.1.3.1.1.4.1.1.2.1.8.1.7.4.3.2.1.3.5.3.9.6.1.15.10.28.1.2.2.12.3.4.1.6.3.4.7.1.3.1.1.3.1.5.3.1.3.2.2.1.1.4.2.1.2.1.2.2.2.4.2.1.2.2.2.4.1.1.1.2.2.1.1.1.1.2.1.1.1.2.2.1.1.2.1.2.1.7.1.2.1.1.1.2.1.1.1.1.2.1.1.3.2.1.1.8.1.1.1.5.2.1.6.5.1.1.1.1.1.2.2.3.1.1.4.1.1.2.2.1.1.4.3.1.2.2.1.2.1.2.3.1.1.2.4.1.1.1.5.1.3.6.3.1.5.2.3.4.1.2.3.1.4.2.1.2.2.2.1.1.1.1.1.1.11.1.3.1.1.2.2.5.2.3.3.5.1.1.1.4.2.1.1.2.5.1.9.4.1.1.3.1.7.1.4.5.1.7.2.1.1.1.2.1.1.1.4.2.1.12.1.1.3.1.2.2.3.1.2.1.1.1.2.1.1.2.1.1.1.1.2.1.3.1.5.1.2.4.3.8.2.2.9.7.2.3.2.1.4.6.1.1.6.1.1; sw_l10m=us; sw_l10org=US"),
    // );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("en-GB,en-US;q=0.7,en;q=0.3"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(
        reqwest::header::UPGRADE_INSECURE_REQUESTS,
        reqwest::header::HeaderValue::from_static("1"),
    );
    headers.insert(
        reqwest::header::CONNECTION,
        reqwest::header::HeaderValue::from_static("keep-alive"),
    );
    headers.insert(
        reqwest::header::REFERER,
        reqwest::header::HeaderValue::from_static(
            "https://int.soccerway.com/search/?q=singapore&module=all",
        ),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-dest"),
        reqwest::header::HeaderValue::from_static("document"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-mode"),
        reqwest::header::HeaderValue::from_static("navigate"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("sec-fetch-site"),
        reqwest::header::HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        reqwest::header::TE,
        reqwest::header::HeaderValue::from_static("trailers"),
    );
    let client = client_builder
        .connection_verbose(true)
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:102.0) Gecko/20100101 Firefox/102.0",
        )
        .cookie_store(true)
        .default_headers(headers);
    client.build()
    // let client = client.build()?;
    // Tricking the anti bot measures by first doing a regular frontpage load and getting the
    // cookies and whatever I guess?
    // let _resp = client
    //     .get("https://int.soccerway.com")
    //     .version(reqwest::Version::HTTP_11)
    //     .send()
    //     .await?;
    // Ok(client)
}

#[derive(Debug)]
pub struct RankingEntry {
    rank: u8,
    team: String,
    _played: u8,
    win: u8,
    draw: u8,
    lose: u8,
    gf: u8,
    ga: u8,
    _gd: i8,
    points: u8,
}

impl RankingEntry {
    fn parse_from_row(row: scraper::ElementRef) -> Option<RankingEntry> {
        // TODO Make this return a Result<> instead of Option<>

        let cell_selector = Selector::parse("td").ok()?;
        let mut cells = row.select(&cell_selector);
        // Need to clean this up, too much repetition
        let rank = cells.next()?.text().next()?.parse().ok()?;
        let team = cells.nth(1)?.text().next()?.to_owned();
        let _played = cells.next()?.text().next()?.parse().ok()?;
        let win = cells.next()?.text().next()?.parse().ok()?;
        let draw = cells.next()?.text().next()?.parse().ok()?;
        let lose = cells.next()?.text().next()?.parse().ok()?;
        let gf = cells.next()?.text().next()?.parse().ok()?;
        let ga = cells.next()?.text().next()?.parse().ok()?;
        let _gd = cells.next()?.text().next()?.parse().ok()?;
        let points = cells.next()?.text().next()?.parse().ok()?;

        Some(RankingEntry {
            rank,
            team,
            _played,
            win,
            draw,
            lose,
            gf,
            ga,
            _gd,
            points,
        })
    }
}

impl std::fmt::Display for RankingEntry {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_euro_group_b() {
        let content = include_str!("euro2021-group-b.html");
        let ranking = parse_ranking(content);
        let belgium = RankingEntry {
            rank: 1,
            team: String::from("Belgium"),
            _played: 0,
            win: 0,
            draw: 0,
            lose: 0,
            gf: 0,
            ga: 0,
            _gd: 0,
            points: 0,
        };
        assert_eq!(ranking[0].rank, belgium.rank);
        assert_eq!(ranking[0].team, belgium.team);
        assert_eq!(ranking[0].gf, belgium.gf);
    }

    #[test]
    fn parse_belgian_playoff() {
        let content = include_str!("be2021-playoffs.html");
        let ranking = parse_ranking(content);
        let anderlecht = RankingEntry {
            rank: 4,
            team: String::from("Anderlecht"),
            _played: 6,
            win: 0,
            draw: 4,
            lose: 2,
            gf: 9,
            ga: 11,
            _gd: -2,
            points: 33,
        };
        assert_eq!(ranking[3].rank, anderlecht.rank);
        assert_eq!(ranking[3].team, anderlecht.team);
        assert_eq!(ranking[3].gf, anderlecht.gf);
    }
}
