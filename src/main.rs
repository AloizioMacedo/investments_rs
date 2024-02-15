use anyhow::Result;

mod config;
mod models;
mod outputs;
mod preprocess;
mod timeseries;

fn main() -> Result<()> {
    preprocess::main()?;
    models::main()?;
    outputs::main()?;

    Ok(())
}
