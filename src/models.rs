use anyhow::Result;
use std::path::Path;

use polars::{
    lazy::dsl::{col, lit},
    prelude::*,
};

use crate::timeseries::TimeSeries;

pub fn load_all_funds() -> Result<DataFrame> {
    let path = Path::new("data/02_preprocessed/funds.csv");

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

pub fn main() -> Result<()> {
    let funds = load_all_funds()?;
    let ts = convert_funds_into_timeseries(funds, "2022-01-01", "2023-01-01");

    for t in ts {
        println!("Ts values: {:?}", t.returns);
    }

    Ok(())
}
