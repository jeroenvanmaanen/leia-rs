use std::{cmp::max, collections::BTreeSet};

use rational::Rational;

#[derive(Debug)]
pub struct Rationals {
    segments: Vec<(u64, Rational)>,
}

impl Rationals {
    fn len(&self) -> u64 {
        self.segments.len() as u64
    }
    fn offer(&mut self, item: (u64, Rational)) {
        self.segments.insert(0, item);
    }
}

#[derive(Debug)]
struct Threshold {
    bound_a: Rational,
    bound_b: Rational,
    count_a: u64,
    count_b: u64,
}

#[derive(Clone, Copy)]
struct IntervalSpec {
    bound: Rational,
    count: u64,
}

struct MappedInterval {
    ma: Rational,
    a: Rational,
    mb: Rational,
    b: Rational,
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
        let thresholds = get_thresholds(w);
        let weight = thresholds[thresholds.len() - 1].count_b;

        if 3 * weight <= 2 * smaller_rationals.len() {
            smaller_rationals
        } else {
            let mapped = map_to_thresholds(thresholds, &boundaries);

            let mut previous = (Rational::zero(), Rational::zero());
            let mut mapped_intervals = vec![];
            for b in &mapped {
                mapped_intervals.push((previous, b));
                previous = *b;
            }
            let mut it = mapped_intervals.iter();
            let first = it.next().unwrap();
            let first_mapped = MappedInterval {
                ma: first.0.0,
                a: first.0.1,
                mb: first.1.0,
                b: first.1.1,
            };
            let max_mapped = it.fold(first_mapped, max_mapped_interval);
            let scaled = scale_interval(Rational::new(1, 3), (max_mapped.a, max_mapped.b));
            let continued_fraction = continued_fraction_expand_r(scaled.0, scaled.1, false, false);
            let extra = continued_fraction_compute(&mut continued_fraction.iter());

            let mut smaller_rationals = smaller_rationals;
            smaller_rationals.offer((w, extra));
            smaller_rationals
        }
    }
}

fn map_to_thresholds(
    thresholds: Vec<Threshold>,
    rationals: &BTreeSet<Rational>,
) -> Vec<(Rational, Rational)> {
    let mut result = vec![];
    let mut it = thresholds.iter();
    let mut threshold = it.next().unwrap();
    for x in rationals {
        let x = *x;
        if x > threshold.bound_b {
            threshold = it.next().unwrap();
        } else if x > threshold.bound_a {
            let width = threshold.bound_b - threshold.bound_a;
            let count = threshold.count_b - threshold.count_a;
            let mapped = (((x - threshold.bound_a) * count) / width) + threshold.count_a;
            result.push((mapped, x));
        } else {
            panic!()
        }
    }
    result
}

fn max_mapped_interval(
    current: MappedInterval,
    next_mapped: &((Rational, Rational), &(Rational, Rational)),
) -> MappedInterval {
    let MappedInterval { ma, mb, .. } = current;
    let ((mc, c), (md, d)) = next_mapped;
    if md - mc > mb - ma {
        MappedInterval {
            ma: *mc,
            a: *c,
            mb: *md,
            b: *d,
        }
    } else {
        current
    }
}

fn scale_interval(r: Rational, interval: (Rational, Rational)) -> (Rational, Rational) {
    let (a, b) = interval;
    let w = b - a;
    let ww = r * w;
    let aa = a + ((w - ww) * Rational::new(1, 2));
    let bb = aa + ww;
    (aa, bb)
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
        let summed = interval_specs.iter().fold(vec![], sum_interval_specs);
        summed.iter().fold(vec![], combine_interval_specs)
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

fn sum_interval_specs(last_list: Vec<IntervalSpec>, interval: &IntervalSpec) -> Vec<IntervalSpec> {
    if last_list.is_empty() {
        vec![
            *interval,
            IntervalSpec {
                bound: Rational::zero(),
                count: 0,
            },
        ]
    } else {
        let mut last_list = last_list;
        let IntervalSpec {
            bound: _,
            count: last_count,
        } = last_list[0];
        let new_head = IntervalSpec {
            bound: interval.bound,
            count: interval.count + last_count,
        };
        last_list.insert(0, new_head);
        last_list
    }
}

fn combine_interval_specs(last_list: Vec<Threshold>, interval: &IntervalSpec) -> Vec<Threshold> {
    if last_list.is_empty() {
        vec![Threshold {
            bound_a: interval.bound,
            bound_b: interval.bound,
            count_a: interval.count,
            count_b: interval.count,
        }]
    } else {
        let mut last_list = last_list;
        let Threshold {
            bound_a: last_bound,
            count_a: last_count,
            ..
        } = last_list[0];
        let new_head = Threshold {
            bound_a: interval.bound,
            bound_b: last_bound,
            count_a: interval.count,
            count_b: last_count,
        };
        last_list.insert(0, new_head);
        last_list
    }
}

fn continued_fraction_compute<'a, I>(items: &'a mut I) -> Rational
where
    I: Iterator<Item = &'a u64>,
{
    let first = items.next().unwrap();
    continued_fraction_compute_rec(*first, items)
}

fn continued_fraction_compute_rec<'a, I>(first: u64, items: &'a mut I) -> Rational
where
    I: Iterator<Item = &'a u64>,
{
    if let Some(item) = items.next() {
        let frac = Rational::one() / continued_fraction_compute_rec(*item, items);
        first + frac
    } else {
        Rational::new(first, 1)
    }
}

fn continued_fraction_expand_r(
    a: Rational,
    b: Rational,
    include_a: bool,
    include_b: bool,
) -> Vec<u64> {
    if a > b {
        continued_fraction_expand_r(b, a, include_b, include_a)
    } else if a < b {
        let ceil_a_rational = a.ceil();
        let floor_b_rational = b.floor();
        let candidate_rational = if include_a || (ceil_a_rational > a) {
            ceil_a_rational
        } else {
            ceil_a_rational + 1
        };
        let candidate = candidate_rational.mixed_fraction().0 as u64;
        if candidate_rational <= floor_b_rational && (include_b || candidate_rational < b) {
            vec![candidate]
        } else {
            let floor_a_rational = a.floor();
            let floor_a = floor_a_rational.mixed_fraction().0 as u64;
            let inverted_upper_bound = Rational::one() / (b - floor_a_rational);
            if a <= floor_a_rational {
                vec![floor_a, int_part(inverted_upper_bound.floor()) + 1]
            } else {
                let mut result = continued_fraction_expand_r(
                    inverted_upper_bound,
                    Rational::one() / (a - floor_a_rational),
                    include_b,
                    include_a,
                );
                result.insert(0, floor_a);
                result
            }
        }
    } else {
        panic!()
    }
}

fn int_part(r: Rational) -> u64 {
    r.mixed_fraction().0 as u64
}

#[cfg(test)]
mod test {
    use super::*;
    use log::info;
    use test_log::test;

    fn test_continued_fraction(
        a: Rational,
        b: Rational,
        include_a: bool,
        include_b: bool,
        expected: Rational,
    ) {
        let continued_fraction = continued_fraction_expand_r(a, b, include_a, include_b);
        let rational = continued_fraction_compute(&mut continued_fraction.iter());
        info!("");
        info!("Interval: {include_a}, {a}, {b}, {include_b}");
        info!("Continued fraction expand: {continued_fraction:?}");
        info!("Corresponding rational: {rational}");
        if include_a {
            assert!(a <= rational);
        } else {
            assert!(a < rational);
        }
        if include_b {
            assert!(rational <= b);
        } else {
            assert!(rational < b);
        }
        assert_eq!(expected, rational);
    }

    #[test]
    fn test_continued_fractions() {
        info!("");
        info!("Test continued fractions");
        test_continued_fraction(
            Rational::new(5, 4),
            Rational::new(1330, 1000),
            false,
            false,
            Rational::new(9, 7),
        );
        test_continued_fraction(
            Rational::new(5, 4),
            Rational::new(1330, 1000),
            true,
            false,
            Rational::new(5, 4),
        );
        test_continued_fraction(
            Rational::new(124, 100),
            Rational::new(5, 4),
            false,
            false,
            Rational::new(36, 29),
        );
        test_continued_fraction(
            Rational::new(1241, 1000),
            Rational::new(5, 4),
            false,
            true,
            Rational::new(5, 4),
        );
    }

    fn test_weight(weight: u64) {
        info!("Thresholds for {weight}: [");
        for threshold in get_thresholds(weight) {
            info!("  {threshold:?}")
        }
        info!("]");
        info!("");
    }

    const TEST_WEIGHTS: &[u64] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 100, 1000];

    #[test]
    fn test_weights() {
        for weight in TEST_WEIGHTS {
            test_weight(*weight);
        }
    }

    fn test_get_rationals(weight: u64) {
        let rationals = get_rationals(weight);
        info!("Rationals for weight: {weight}");
        for (w, r) in rationals.segments {
            info!("  ({w}, {r})");
        }
        info!("");
    }

    #[test]
    fn test_get_rationals_for_weights() {
        for weight in TEST_WEIGHTS {
            test_get_rationals(*weight);
        }
    }
}
