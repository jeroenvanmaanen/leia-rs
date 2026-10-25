use anyhow::Result;
use compute_lattice::get_rationals;
use log::info;

fn main() -> Result<()> {
    env_logger::init();
    info!("Compute lattice");
    let rationals = get_rationals(16);
    info!("Rationals: [{rationals:?}]");
    Ok(())
}
