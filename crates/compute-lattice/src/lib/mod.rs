use anyhow::Result;
use log::info;

#[derive(Debug)]
struct TruncatedLattice {
    weight: u64,
    segments: Vec<(u64, ())>,
}

pub fn compute_lattice(_rank: i64) -> Result<()> {
    let lattice = TruncatedLattice {
        weight: 0,
        segments: vec![],
    };
    let lattice = extend_lattice(lattice, 1);
    info!("Lattice: [{lattice:?}]");
    Ok(())
}

fn extend_lattice(previous_lattice: TruncatedLattice, w: u64) -> TruncatedLattice {
    let mut previous_lattice = previous_lattice;
    previous_lattice.segments.push((w, ()));
    previous_lattice
}
