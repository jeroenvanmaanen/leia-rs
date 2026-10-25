use anyhow::Result;
use compute_lattice::compute_lattice;
use log::info;

fn main() -> Result<()> {
    env_logger::init();
    info!("Compute lattice");
    compute_lattice(16)?;
    Ok(())
}
