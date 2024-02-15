use polars::prelude::*;
use std::path::Path;

use anyhow::{anyhow, Result};

pub fn main() -> Result<()> {
    process_funds()?;
    process_cdi()?;

    Ok(())
}

pub fn get_month(s: &str) -> Result<&str> {
    match s {
        "Jan" => Ok("01"),
        "Fev" => Ok("02"),
        "Mar" => Ok("03"),
        "Abr" => Ok("04"),
        "Mai" => Ok("05"),
        "Jun" => Ok("06"),
        "Jul" => Ok("07"),
        "Ago" => Ok("08"),
        "Set" => Ok("09"),
        "Out" => Ok("10"),
        "Nov" => Ok("11"),
        "Dez" => Ok("12"),
        _ => Err(anyhow!("Can't parse this month")),
    }
}

pub fn process_cdi() -> Result<()> {
    let cdi_path = Path::new("data/01_raw/cdi.csv");

    let mut df = CsvReader::from_path(cdi_path)?.finish()?;

    _ = df.drop_in_place("Acumulado")?;

    let vals: Vec<String> = Vec::new();
    let mut df = df.melt(["Ano/Mês"], vals)?;

    df.rename("variable", "month")?;
    df.rename("value", "values")?;

    df.apply("values", |x| {
        x.str()
            .expect("Column 'values' should be strings with commas instead of dots")
            .into_iter()
            .map(|x| {
                let x = x.expect("Column 'values' should not have missing values");
                x.replace(',', ".")
            })
            .collect::<StringChunked>()
            .into_series()
            .to_float()
            .expect("Should be able to convert 'values' to floats after replacing ',' with '.'")
    })?;

    df.apply("values", |x| {
        x.f64()
            .expect("Column 'values' should be floats after converison")
            .apply(|x| x.map(|y| y / 100.0))
    })?;

    let cast_year = df["Ano/Mês"].cast(&DataType::String)?;
    let year = cast_year
        .str()
        .expect("Column 'Ano/Mês' should be strings after data cast.");
    let n = year.len();

    let month = df["month"]
        .str()
        .expect("Column 'month' should be strings representing months, e.g. 'Jan', 'Fev' etc.")
        .apply(|x| {
            Some(std::borrow::Cow::from(
                "-".to_string()
                    + get_month(x.expect("Column 'month' should not have missing values"))
                        .expect("Column 'month' should be a month such as 'Jan', 'Fev' etc."),
            ))
        });

    let days: Series = std::iter::repeat("-01").take(n).collect();
    let days = days.str().expect("Series 'days' should consist of strings");

    let dt = year.clone() + month.clone() + days.clone();
    let dt = dt.with_name("dt");

    df.with_column(dt)?;
    _ = df.drop_in_place("Ano/Mês")?;
    _ = df.drop_in_place("month")?;

    df.sort_in_place(["dt"], vec![false], true)?;

    let path = Path::new("data/02_preprocessed/cdi.csv");
    let file = std::fs::File::create(path)?;

    CsvWriter::new(file).finish(&mut df)?;

    Ok(())
}

pub fn process_funds() -> Result<()> {
    let raw_path = Path::new("data/01_raw");
    let preprocessed_path = Path::new("data/02_preprocessed");

    let mut dataframes = Vec::new();

    for file in raw_path.join("fundos").read_dir()? {
        let csv_file = file?;

        let mut df = CsvReader::from_path(csv_file.path())?
            .has_header(true)
            .finish()?;

        let path = csv_file.path();
        let name = path.file_stem().ok_or(anyhow!("File name not found"))?;
        let file_name = name.to_str().ok_or(anyhow!("Invalid UTF8 for file name"))?;

        let file_name = file_name.replacen('_', "/", 1);
        let (cnpj, year) = file_name
            .split_once('_')
            .ok_or(anyhow!("Invalid file name. Couldn't split CNPJ and date."))?;

        _ = df.drop_in_place("");
        _ = df.drop_in_place("Acumulado");

        let mut transposed = df
            .transpose(Some("month"), None)
            .expect("Could not transpose");

        transposed
            .rename("column_0", "values")
            .expect("Could not rename");

        let months = transposed["month"].clone();
        let s = months.str().expect("Months should be strings");

        let months_as_numbers = s.apply(|x| {
            let month = get_month(x.expect("Months should be strings"))
                .expect("Months should be 3-letter and in Portuguese");

            Some(std::borrow::Cow::from(month))
        });

        let len = months_as_numbers.len();

        let year: Series = std::iter::repeat(year.to_string() + "-")
            .take(len)
            .collect();
        let year = year.str()?;

        let day: Series = std::iter::repeat("-01").take(len).collect();
        let day = day.str()?;

        let dt = year.clone() + months_as_numbers + day.clone();
        let dt = Series::from(dt);
        let dt = dt.with_name("dt");

        transposed.with_column(dt)?;
        _ = transposed.drop_in_place("month")?;

        let cnpj: Series = std::iter::repeat(cnpj).take(len).collect();
        let cnpj = cnpj.with_name("CNPJ_Fundo");

        transposed.with_column(cnpj)?;

        transposed.apply("values", |series| {
            series
                .str()
                .expect("'values' column was not a string.")
                .into_iter()
                .map(|value| value.map(|x| x.replace(',', ".")))
                .collect::<StringChunked>()
                .into_series()
                .to_float()
                .expect("Could not convert to float")
        })?;

        transposed.apply("values", |series| {
            series
                .f64()
                .expect("'values' should be floats by now.")
                .apply(|entry| entry.map(|x| x / 100.0))
                .into_series()
        })?;

        dataframes.push(transposed);
    }

    let mut df = dataframes
        .into_iter()
        .reduce(|acc, df| {
            acc.vstack(&df)
                .expect("Should be able to vertically stack dataframes")
        })
        .expect("Should have more than one dataframe");

    df.sort_in_place(["CNPJ_Fundo", "dt"], vec![false, false], true)?;

    let path = Path::new(preprocessed_path);

    let file = std::fs::File::create(path.join("funds.csv"))?;

    CsvWriter::new(file).finish(&mut df)?;

    Ok(())
}
