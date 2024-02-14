use anyhow::{anyhow, Result};
use std::path::Path;

use polars::prelude::*;

fn get_month(s: &str) -> Result<i32> {
    match s {
        "Jan" => Ok(1),
        "Fev" => Ok(2),
        "Mar" => Ok(3),
        "Abr" => Ok(4),
        "Mai" => Ok(5),
        "Jun" => Ok(6),
        "Jul" => Ok(7),
        "Ago" => Ok(8),
        "Set" => Ok(9),
        "Out" => Ok(10),
        "Nov" => Ok(11),
        "Dez" => Ok(12),
        _ => Err(anyhow!("Can't parse this month")),
    }
}

fn main() -> Result<()> {
    let raw_path = Path::new("data/01_raw");

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
            let month = get_month(x.unwrap()).expect("Months should be 3-letter and in Portuguese");
            let month = month.to_string();
            let month = format!("{:0>2}", month);

            Some(std::borrow::Cow::from(month))
        });

        let len = months_as_numbers.len();

        let year: Series = std::iter::repeat(year.to_string() + "-")
            .take(len)
            .collect();
        let year = year.str().unwrap();

        let day: Series = std::iter::repeat("-01").take(len).collect();
        let day = day.str().unwrap();

        let dt = year.clone() + months_as_numbers + day.clone();
        let dt = Series::from(dt);
        let dt = dt.with_name("dt");

        transposed.with_column(dt)?;
        _ = transposed.drop_in_place("month")?;

        let cnpj: Series = std::iter::repeat(cnpj).take(len).collect();
        let cnpj = cnpj.with_name("CNPJ_Fundo");

        transposed.with_column(cnpj)?;

        transposed.apply("values", |x| {
            x.str()
                .expect("'values' column was not a string.")
                .into_iter()
                .map(|value| value.map(|x| x.replace(',', ".")))
                .collect::<StringChunked>()
                .into_series()
                .to_float()
                .expect("Could not convert to float")
        })?;

        println!("DF: {}", transposed);
        dataframes.push(transposed);
    }

    Ok(())
}
