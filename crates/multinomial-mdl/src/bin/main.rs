use std::env;

use anyhow::Result;
use clap::Parser;
use log::info;
use multinomial_mdl::{Counts, MaximumLikelihood, MultinomialModel, new_counts};

use crate::Command::ENCODE;

/// Find the MDL-encoding for a multinomial distribution
#[derive(Parser, Debug)]
enum Command {
    /// Encode the given distribution
    ENCODE { counts: Vec<u64> },
}

fn main() -> Result<()> {
    env_logger::init();
    println!(
        "
Multinomial MDL
===============
"
    );
    let command = Command::parse_from(env::args());
    println!("Command: [{command:?}]");

    info!("Multinomial MDL");
    match command {
        ENCODE {
            counts: count_values,
            ..
        } => {
            let counts = new_counts(count_values);
            if let Some(ml) = counts.as_maximum_likelihood() {
                let model = ml.maximum_likelihood_model();
                info!("Model: [{model:?}]");
                let description_length = model.evaluate(&counts)?;
                info!("Description length of model plus data given model = {description_length}");
            }
        }
    }
    Ok(())
}
