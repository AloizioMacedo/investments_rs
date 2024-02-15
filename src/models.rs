use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

use polars::{
    lazy::dsl::{col, lit},
    prelude::*,
};

use crate::config::get_config;
use crate::timeseries::{AllTimeSeries, TimeSeries};

pub fn load_all_funds() -> Result<DataFrame> {
    let path = Path::new("data/02_preprocessed/funds.csv");

    Ok(CsvReader::from_path(path)?.has_header(true).finish()?)
}

pub fn load_cdi() -> Result<DataFrame> {
    let path = Path::new("data/02_preprocessed/cdi.csv");

    Ok(CsvReader::from_path(path)?.has_header(true).finish()?)
}

pub fn convert_funds_into_timeseries(
    df: DataFrame,
    from_date: &str,
    to_date: &str,
) -> Vec<TimeSeries> {
    let names = df["CNPJ_Fundo"]
        .unique()
        .expect("Column 'CNPJ_Fundo' should be present");

    let cnpjs = names.str().expect("Column 'CNPJ_Fundo' should be strings");

    let lazy = df.lazy();

    cnpjs
        .into_iter()
        .map(|name| {
            let name = name.unwrap();
            let df = lazy
                .clone()
                .filter(col("CNPJ_Fundo").eq(lit(name)))
                .filter(col("dt").gt_eq(lit(from_date)))
                .filter(col("dt").lt_eq(lit(to_date)));

            let df = df.collect().expect("Filtering should be possible");

            let values = df["values"]
                .f64()
                .unwrap()
                .into_iter()
                .map(|x| x.unwrap())
                .collect();

            TimeSeries::new(name.to_string(), values)
        })
        .collect()
}

pub fn convert_cdi_into_timeseries(df: DataFrame, from_date: &str, to_date: &str) -> TimeSeries {
    let lazy = df.lazy();

    let df = lazy
        .clone()
        .filter(col("dt").gt_eq(lit(from_date)))
        .filter(col("dt").lt_eq(lit(to_date)));

    let df = df.collect().expect("Filtering should be possible");

    let values = df["values"]
        .f64()
        .unwrap()
        .into_iter()
        .map(|x| x.unwrap())
        .collect();

    TimeSeries::new("_cdi".to_string(), values)
}

pub fn main() -> Result<()> {
    let config = get_config();
    let funds = load_all_funds()?;

    let timeseries = convert_funds_into_timeseries(
        funds,
        &config.portfolio.from_date,
        &config.portfolio.to_date,
    );

    let all_timeseries = AllTimeSeries { timeseries };

    let jsonified_ts = serde_json::to_string(&all_timeseries)?;
    let path = Path::new("data/03_models/models.json");

    std::fs::write(path, jsonified_ts)?;

    let cdi = load_cdi()?;
    let cdi_ts =
        convert_cdi_into_timeseries(cdi, &config.portfolio.from_date, &config.portfolio.to_date);

    let jsonified_ts = serde_json::to_string(&cdi_ts)?;
    let path = Path::new("data/03_models/cdi.json");

    std::fs::write(path, jsonified_ts)?;

    Ok(())
}
