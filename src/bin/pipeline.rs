use anyhow::Result;

mod models;
mod outputs;
mod preprocess;

fn main() -> Result<()> {
    preprocess::main()?;
    models::main()?;
    outputs::main()?;

    Ok(())
}
