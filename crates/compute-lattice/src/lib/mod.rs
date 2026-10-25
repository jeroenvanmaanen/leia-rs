use std::{cmp::max, collections::BTreeSet};

use rational::Rational;

#[derive(Debug)]
pub struct Rationals {
    segments: Vec<(u64, Rational)>,
}

struct Threshold {
    bound_a: Rational,
    bound_b: Rational,
    count_a: u64,
    count_b: u64,
}

struct IntervalSpec {
    bound: Rational,
    count: u64,
}

pub fn get_rationals(w: u64) -> Rationals {
    let initial = Rationals {
        segments: vec![(1, Rational::one())],
    };
    update_rationals(1, initial, w)
}

fn update_rationals(previous_weight: u64, previous_rationals: Rationals, w: u64) -> Rationals {
    if w <= previous_weight {
        previous_rationals
    } else {
        let smaller_rationals = update_rationals(previous_weight, previous_rationals, w - 1);
        let boundaries = (&smaller_rationals)
            .segments
            .iter()
            .map(|p| p.1)
            .collect::<BTreeSet<Rational>>();
        let mut previous = Rational::zero();
        let mut intervals = vec![];
        for b in boundaries {
            intervals.push((previous, b));
            previous = b;
        }
        let thresholds = get_thresholds(w);
        let mut previous_lattice = smaller_rationals;
        previous_lattice.segments.push((w, Rational::one()));
        previous_lattice
    }
}

fn get_thresholds(w: u64) -> Vec<Threshold> {
    if w < 4 {
        let threshold = Threshold {
            bound_a: Rational::zero(),
            bound_b: Rational::one(),
            count_a: 0,
            count_b: w,
        };
        vec![threshold]
    } else {
        let square_root = (w as f64).sqrt();
        let hyper_root = square_root.sqrt();
        let intervals = max(1, square_root.floor() as u64);
        let spacing = Rational::new(w, intervals);
        let mut boundaries = vec![];
        let mut b = Rational::zero();
        while b < w {
            boundaries.push(b);
            b += spacing;
        }
        let interval_specs: Vec<IntervalSpec> = boundaries
            .into_iter()
            .map(|b| get_interval_spec(w, spacing, square_root, hyper_root, b))
            .collect();
        //        interval_specs.iter().fold(init, f)
        vec![]
    }
}

fn get_interval_spec(
    weight: u64,
    spacing: Rational,
    square_root: f64,
    hyper_root: f64,
    boundary: Rational,
) -> IntervalSpec {
    if boundary == Rational::zero() {
        IntervalSpec {
            bound: spacing / weight,
            count: max(1, spacing.floor().mixed_fraction().0 as u64),
        }
    } else {
        let square_root_of_boundary = boundary.decimal_value() / square_root;
        let sub_intervals = max(1, (hyper_root / square_root_of_boundary).floor() as u64);
        IntervalSpec {
            bound: (boundary + spacing) / weight,
            count: sub_intervals,
        }
    }
}
