use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub funds_filters: FundsFilters,
    pub portfolio: Portfolio,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundsFilters {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub volatility_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Portfolio {
    pub number_of_funds: usize,
    pub from_date: String,
    pub to_date: String,
    pub split_granularity: f64,
}

pub fn get_config() -> Config {
    let config = std::fs::read_to_string("config/config.toml")
        .expect("'config.toml' should be present inside config folder.");

    toml::from_str(&config).expect("Config should be a toml file with proper attributes")
}
