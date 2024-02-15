use anyhow::Result;
use indicatif::ProgressBar;
use itertools::Itertools;
use std::path::Path;

use crate::{
    config::get_config,
    timeseries::{AllTimeSeries, Portfolio, TimeSeries},
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

fn load_cdi() -> TimeSeries {
    let path = Path::new("data/03_models/cdi.json");
    let timeseries = std::fs::read_to_string(path).unwrap();

    serde_json::from_str(&timeseries).unwrap()
}

fn get_possible_splits() -> impl Iterator<Item = Vec<f64>> {
    let config = get_config();

    let min_gran = config.portfolio.split_granularity;
    let n_funds = config.portfolio.number_of_funds;

    let total = (1.0 / min_gran) as usize;

    let granularity = (0..total).map(|x| x as f64 * min_gran).collect_vec();

    std::iter::repeat(granularity)
        .take(n_funds - 1)
        .multi_cartesian_product()
        .map(|mut x| {
            x.push(1.0 - x.iter().sum::<f64>());
            x
        })
}

fn get_statistics_from_splits(
    risk_free: &TimeSeries,
    funds: &[TimeSeries],
    possible_splits: impl Iterator<Item = Vec<f64>>,
) -> Statistics {
    let mut splits = Vec::new();
    let mut volatilities = Vec::new();
    let mut average_returns = Vec::new();
    let mut returns_at_end = Vec::new();
    let mut sharpe_ratios = Vec::new();

    let possible_splits = possible_splits.collect_vec();

    let pb = ProgressBar::new(possible_splits.len() as u64);
    for possible_split in possible_splits {
        let p = Portfolio::new(funds.to_vec(), possible_split.clone()).unwrap();

        volatilities.push(p.std());
        average_returns.push(p.average());
        returns_at_end.push(p.calculate_value_at_end(1.0));
        sharpe_ratios.push(p.sharpe_ratio(risk_free));
        splits.push(possible_split);
        pb.inc(1);
    }
    pb.finish();

    Statistics {
        splits,
        volatilities,
        average_returns,
        returns_at_end,
        sharpe_ratios,
    }
}

pub fn main() -> Result<()> {
    let funds = load_timeseries();
    let cdi = load_cdi();
    let possible_splits = get_possible_splits();

    _ = get_statistics_from_splits(&cdi, &funds, possible_splits);

    Ok(())
}
