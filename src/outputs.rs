use anyhow::{anyhow, Result};
use std::path::Path;

use crate::timeseries::{AllTimeSeries, TimeSeries};

fn load_timeseries() -> Vec<TimeSeries> {
    let path = Path::new("data/03_models/models.json");
    let timeseries = std::fs::read_to_string(path).unwrap();

    let all_ts: AllTimeSeries = serde_json::from_str(&timeseries).unwrap();

    all_ts.timeseries
}

pub fn main() -> Result<()> {
    let ts = load_timeseries();

    Ok(())
}
