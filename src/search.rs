use crate::generic_structs::*;
use chrono::prelude::*;

impl Football {
    /// Splits string into pieces, only keeps games for which every piece is matched by either
    /// country, competition, or teams
    pub fn query(&self, query: &str) -> Football {
        let query: Vec<_> = query
            .split(|c: char| !c.is_ascii_alphabetic())
            .map(|word| word.to_lowercase())
            .collect();
        let mut games = Football { countries: vec![] };
        for country in &self.countries {
            let mut filteredcompetitions = vec![];
            for competition in &country.competitions {
                let filteredgames: Vec<_> = competition
                    .games
                    .iter()
                    .filter(|game| {
                        query.iter().all(|word| {
                            country.name.to_lowercase().contains(word)
                                || competition.name.to_lowercase().contains(word)
                                || game.home_team.to_lowercase().contains(word)
                                || game.away_team.to_lowercase().contains(word)
                        })
                    })
                    .cloned()
                    .collect();
                if !filteredgames.is_empty() {
                    filteredcompetitions.push(Competition {
                        name: competition.name.to_owned(),
                        games: filteredgames,
                    });
                }
            }
            if !filteredcompetitions.is_empty() {
                games.countries.push(Country {
                    name: country.name.to_owned(),
                    competitions: filteredcompetitions,
                });
            }
        }
        games
    }

    // TODO: I want both "women world cup" and "world cup women" to match the same manner.
    // Idea: split up and see if I can just add the match number for each to get something
    // meaningful? (Perhaps this is more something for the bitap library side)
    pub fn fuzzy_query(&self, query: &str) -> Vec<(f64, Country, Competition, Game)> {
        let query = query.to_lowercase();
        let bitap = bitap::Bitap::new().distance(100_000).threshold(0.3);
        let mut result = vec![];
        for country in &self.countries {
            for competition in &country.competitions {
                for game in &competition.games {
                    let fullstr = format!(
                        "{} {} {} {}",
                        country.name, competition.name, game.home_team, game.away_team
                    )
                    .to_lowercase();
                    let matcher = bitap.bitap(&fullstr, &query);
                    if matcher.is_match {
                        result.push((
                            matcher.score,
                            country.clone(),
                            competition.clone(),
                            game.clone(),
                        ));
                    }
                }
            }
        }
        result.sort_by(|(score1, _, _, _), (score2, _, _, _)| score1.partial_cmp(score2).unwrap());
        result
    }

    // TODO: I want both "women world cup" and "world cup women" to match the same manner.
    // Idea: split up and see if I can just add the match number for each to get something
    // meaningful? (Perhaps this is more something for the bitap library side)
    pub fn mixed_query(&self, query: &str) -> Vec<(f64, Country, Competition, Game)> {
        let query: Vec<_> = query
            .split(|c: char| !c.is_ascii_alphabetic())
            .map(|word| word.to_lowercase())
            .collect();
        let bitap = bitap::Bitap::new().distance(100_000).threshold(0.45);
        let mut result = vec![];
        for country in &self.countries {
            for competition in &country.competitions {
                for game in &competition.games {
                    let fullstr = format!(
                        "{} {} {} {}",
                        country.name, competition.name, game.home_team, game.away_team
                    )
                    .to_lowercase();
                    // We sum scores, which makes query with more spaces be less of a match.
                    // TODO Use average score?
                    let mut score = Some((0.0, 0));
                    for word in &query {
                        let matcher = bitap.bitap(&fullstr, &word);
                        if !matcher.is_match {
                            score = None;
                            break;
                        }
                        score = score.and_then(|(s, ctr)| Some((s + matcher.score, ctr + 1)));
                    }
                    if let Some((s, ctr)) = score {
                        result.push((
                            s / ctr as f64,
                            country.clone(),
                            competition.clone(),
                            game.clone(),
                        ));
                    }
                }
            }
        }
        result.sort_by(|(score1, _, _, _), (score2, _, _, _)| score1.partial_cmp(score2).unwrap());
        result
    }

    pub fn number_of_games(&self) -> usize {
        let mut ctr = 0;
        for country in &self.countries {
            for competition in &country.competitions {
                ctr += competition.games.len();
            }
        }
        ctr
    }

    // TODO Can I expand this to include basic querying too? Would need different functions to
    // apply to country and competition then though...
    pub fn generic_filter(&self, predicate: impl Fn(&&Game) -> bool + Copy) -> Football {
        let mut games = Football { countries: vec![] };

        for country in &self.countries {
            let mut filteredcompetitions = vec![];
            for competition in &country.competitions {
                let filteredgames: Vec<_> = competition
                    .games
                    .iter()
                    .filter(predicate)
                    .cloned()
                    .collect();
                if !filteredgames.is_empty() {
                    filteredcompetitions.push(Competition {
                        name: competition.name.to_owned(),
                        games: filteredgames,
                    });
                }
            }
            if !filteredcompetitions.is_empty() {
                games.countries.push(Country {
                    name: country.name.to_owned(),
                    competitions: filteredcompetitions,
                });
            }
        }

        games
    }

    /// To help with relevance, show games in a sliding window around the current point in time.
    /// Provide the numbers of hours to include. E.g., 10 and 16 will give games that started up to
    /// 10 hrs earlier up to games that will start in the next 16 hrs.
    pub fn sliding_window(&self, hours_before: u8, hours_after: u8) -> Football {
        let now = Utc::now();
        self.generic_filter(|game| {
            game.start_time <= (now + chrono::Duration::hours(hours_after.into()))
                && game.start_time >= (now - chrono::Duration::hours(hours_before.into()))
        })
    }

    pub fn today(&self) -> Football {
        let now = Utc::now();
        self.generic_filter(|game| game.start_time.ordinal() == now.ordinal())
    }

    pub fn tomorrow(&self) -> Football {
        let now = Utc::now();
        // TODO: Will fail on 31 December.
        self.generic_filter(|game| game.start_time.ordinal() == now.ordinal() + 1)
    }

    pub fn yesterday(&self) -> Football {
        let now = Utc::now();
        // TODO: Will fail on 1 January.
        self.generic_filter(|game| game.start_time.ordinal() == now.ordinal() - 1)
    }

    // TODO: This is more status than time so this module is badly named
    pub fn ended(&self) -> Football {
        self.generic_filter(|game| game.status == GameStatus::Ended)
    }

    pub fn live(&self) -> Football {
        self.generic_filter(|game| match game.status {
            GameStatus::Ongoing(_) => true,
            _ => false,
        })
    }

    pub fn upcoming(&self) -> Football {
        self.generic_filter(|game| game.status == GameStatus::Upcoming)
    }

    /// Only keep games played in country. Empty if nothing
    pub fn country(&self, country_name: &str) -> Self {
        let filtered_countries = self
            .countries
            .iter()
            .filter(|country| country.name.eq_ignore_ascii_case(country_name))
            .cloned();
        Self {
            countries: filtered_countries.collect(),
        }
    }

    /// Only keep games played in competition. Empty if nothing
    pub fn competition(&self, competition_name: &str) -> Self {
        let filtered_countries = self.countries.iter().filter_map(|country| {
            let filtered_comps = country
                .competitions
                .iter()
                .filter(|competition| competition.name.eq_ignore_ascii_case(competition_name))
                .cloned();
            let comps: Vec<_> = filtered_comps.collect();
            if !comps.is_empty() {
                Some(Country {
                    competitions: comps,
                    name: country.name.clone(),
                })
            } else {
                None
            }
        });

        Self {
            countries: filtered_countries.collect(),
        }
    }
}
