mod bin;

use anyhow::Result;
use bin::{models, outputs, preprocess};

fn main() -> Result<()> {
    preprocess::main()?;
    models::main()?;
    outputs::main()?;

    Ok(())
}
