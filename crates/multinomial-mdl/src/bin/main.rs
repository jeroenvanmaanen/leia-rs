use std::env;

use anyhow::Result;
use clap::Parser;
use log::info;
use multinomial_mdl::{Distribution, new_distribution};

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
        ENCODE { counts, .. } => {
            let distribution = new_distribution(counts);
            let encoded = distribution.encode();
            info!("Encoded: [{encoded:?}]");
        }
    }
    Ok(())
}
