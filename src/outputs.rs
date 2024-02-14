use anyhow::Result;
use itertools::Itertools;
use std::path::Path;

use crate::{
    config::get_config,
    timeseries::{AllTimeSeries, TimeSeries},
};

struct Statistics {
    splits: Vec<Vec<f64>>,
    volatilities: Vec<f64>,
    average_returns: Vec<f64>,
    returns_at_end: Vec<f64>,
    sharpe_ratios: Vec<f64>,
}

fn load_timeseries() -> Vec<TimeSeries> {
    let path = Path::new("data/03_models/models.json");
    let timeseries = std::fs::read_to_string(path).unwrap();

    let all_ts: AllTimeSeries = serde_json::from_str(&timeseries).unwrap();

    all_ts.timeseries
}

fn get_possible_splits() -> impl Iterator<Item = Vec<f64>> {
    let config = get_config();

    let min_gran = config.portfolio.split_granularity;
    let n_funds = config.portfolio.number_of_funds;

    let total = (1.0 / min_gran) as usize;

    let granularity = (0..total).map(|x| x as f64 * min_gran).collect_vec();

    std::iter::repeat(granularity)
        .take(n_funds)
        .multi_cartesian_product()
        .filter(|x| x.iter().sum::<f64>() == 1.0)
}

pub fn main() -> Result<()> {
    let ts = load_timeseries();

    Ok(())
}
