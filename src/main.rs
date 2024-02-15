mod bin;

use anyhow::Result;
use bin::{outputs, preprocess, timeseries};

fn main() -> Result<()> {
    preprocess::main()?;
    timeseries::main()?;
    outputs::main()?;

    Ok(())
}
