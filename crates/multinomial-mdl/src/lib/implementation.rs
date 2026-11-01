use std::collections::{BTreeSet, VecDeque};
use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use log::{info, trace};
use regex::Regex;

use crate::implementation::DistributionDescription::Split;
use crate::implementation::DistributionDescription::Uniform;

use super::Description;
use super::Distribution as DistributionTrait;

enum DistributionDescription {
    Uniform(usize),
    Split {
        p: Probability,
        split: BitString,
        probable: Box<DistributionDescription>,
        improbable: Box<DistributionDescription>,
    },
}

impl DistributionDescription {
    fn evaluate(&self, distribution: &Distribution, fraction: f64) -> Result<f64> {
        if fraction < 0.0 || fraction > 1.0 {
            return Err(anyhow!("Invalid fraction: [{fraction}]"));
        }
        let dm = distribution.counts.len();
        match self {
            Uniform(m) => {
                if *m != dm {
                    return Err(anyhow!("Mismatch: [{m}] != [{dm}]"));
                }
                trace!("Evaluate uniform: {distribution:?}");
                if dm == 0 {
                    return Ok(0.0);
                }
                if dm == 1 {
                    let avg_length = fraction.log2();
                    return Ok(distribution.counts[0] as f64 * avg_length);
                }
                let p = fraction / (dm as f64);
                let avg_length = p.log2();
                let n: u64 = distribution.counts.iter().sum();
                let model_length = 1.0;
                Ok(model_length + (n as f64) * avg_length)
            }
            Split {
                p: probability,
                split,
                probable,
                improbable,
            } => {
                let m = split.length() as usize;
                if m != dm {
                    return Err(anyhow!("Mismatch: [{m}] != [{dm}]"));
                }
                let numerator = probability.numerator.0 as f64;
                let denominator = (probability.numerator.0 + probability.weight.0) as f64;
                let p = fraction * numerator / denominator;
                let mut probable_counts = vec![];
                let mut improbable_counts = vec![];
                for (b, c) in split.bits().zip(distribution.counts.iter()) {
                    if b {
                        probable_counts.push(*c);
                    } else {
                        improbable_counts.push(*c);
                    }
                }
                let probable_distribution = Distribution::new(probable_counts);
                let improbable_distribution = Distribution::new(improbable_counts);
                trace!(
                    "Evaluate split: {probability:?} {probable_distribution:?} {improbable_distribution:?}"
                );
                let probable_length = probable.evaluate(&probable_distribution, p)?;
                let improbable_length = improbable.evaluate(&improbable_distribution, 1.0 - p)?;

                let model_length = 1 + probability.length() + split.length();
                Ok((model_length as f64) + probable_length + improbable_length)
            }
        }
    }
}

impl Description for DistributionDescription {
    fn bits(&self) -> impl Iterator<Item = bool> {
        match self {
            Uniform(m) if *m < 2 => {
                let empty = vec![];
                bit_vec_bits(&empty)
            }
            Uniform(_) => {
                let single = vec![BitString::new(0, 1)];
                bit_vec_bits(&single)
            }
            Split { split, .. } => {
                let mut result: Vec<BitString> = vec![];
                result.push(BitString::new(1, 1));
                result.push(split.clone());

                bit_vec_bits(&result)
            }
        }
    }

    fn length(&self) -> u64 {
        todo!()
    }
}

impl Debug for DistributionDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uniform(m) if *m < 2 => {
                f.debug_tuple("Uniform").field(m).finish()?;
                f.write_str(" ''")
            }
            Self::Uniform(m) => {
                f.debug_tuple("Uniform").field(m).finish()?;
                f.write_str(" '0'")
            }
            Self::Split {
                p,
                split,
                probable,
                improbable,
            } => {
                f.write_str("Split['1 ")?;
                f.write_fmt(format_args!("s={:?}", split))?;
                f.write_fmt(format_args!("' p={:?}", p))?;
                f.write_fmt(format_args!(" Prob{{{probable:?}}}"))?;
                f.write_fmt(format_args!(" Imp{{{improbable:?}}}"))
            }
        }
    }
}

/// p = n / (n + w) \in [0, 1]
struct Probability {
    numerator: PrefixFree,
    weight: PrefixFree,
}

impl Probability {
    fn new(n: u64, w: u64) -> Probability {
        let numerator = PrefixFree(n);
        let weight = PrefixFree(w);
        Probability { numerator, weight }
    }
}

impl Description for Probability {
    fn bits(&self) -> impl Iterator<Item = bool> {
        self.numerator.bits().chain(self.weight.bits())
    }

    fn length(&self) -> u64 {
        self.numerator.length() + self.weight.length()
    }
}

impl Debug for Probability {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "({}/{}) n={:?} w={:?}",
            self.numerator.0,
            self.numerator.0 + self.weight.0,
            self.numerator,
            self.weight,
        ))
    }
}

struct PrefixFree(u64);

impl PrefixFree {
    fn to_bit_strings(&self) -> Vec<BitString> {
        let bigger = self.0 + 1;
        let mut length = bigger.ilog2();
        trace!("Bigger: {bigger}: ({length})");
        let mut bit_strings = vec![BitString {
            bits: bigger as u128,
            count: (length + 1) as u8,
        }];
        while length > 0 {
            let length_of_length = length.ilog2();
            trace!("Length of length: {length}: ({length_of_length})");
            let leading_bit = 1 << length_of_length;
            let length_string = BitString {
                bits: (length ^ leading_bit) as u128,
                count: (length_of_length + 1) as u8,
            };
            bit_strings.insert(0, length_string);
            length = length_of_length;
        }
        bit_strings
    }
}

impl Description for PrefixFree {
    fn bits(&self) -> impl Iterator<Item = bool> {
        let v = self.to_bit_strings();
        bit_vec_bits(&v)
    }
    fn length(&self) -> u64 {
        let bit_strings = self.to_bit_strings();
        bit_strings.length()
    }
}

impl Debug for PrefixFree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        static FIRST_RE: LazyLock<Regex> =
            LazyLock::new(|| Regex::new("^(?<f>.)").expect("valid regex"));
        f.write_str("'")?;
        let mut bit_strings = self.to_bit_strings();
        if let Some(last) = bit_strings.pop() {
            for bit_string in bit_strings {
                let formatted = format!("{bit_string:?}");
                let annotated = FIRST_RE.replace(&formatted, "${f}¹");
                f.write_fmt(format_args!("{annotated} "))?
            }
            f.write_fmt(format_args!("{last:?}"))?;
        }
        f.write_str("'")?;
        Ok(())
    }
}

struct BitVecIterator {
    bit_iterators: VecDeque<BitIterator>,
    current: BitIterator,
    closed: bool,
}

impl Iterator for BitVecIterator {
    type Item = bool;
    fn next(&mut self) -> Option<bool> {
        if self.closed {
            return None;
        }
        loop {
            if let Some(b) = self.current.next() {
                return Some(b);
            }
            if let Some(it) = self.bit_iterators.pop_front() {
                self.current = it;
            } else {
                self.closed = true;
                return None;
            }
        }
    }
}

fn bit_vec_bits(bit_strings: &Vec<BitString>) -> BitVecIterator {
    let mut bit_iterators = VecDeque::new();
    for bit_string in bit_strings.iter() {
        bit_iterators.push_back(bit_string.bit_iterator());
    }
    let first = BitIterator { bits: 0, mask: 0 };
    BitVecIterator {
        bit_iterators,
        current: first,
        closed: false,
    }
}

impl Description for Vec<BitString> {
    fn bits(&self) -> impl Iterator<Item = bool> {
        bit_vec_bits(self)
    }

    fn length(&self) -> u64 {
        self.iter().map(BitString::length).sum()
    }
}

#[derive(Clone, Copy)]
struct BitString {
    bits: u128,
    count: u8,
}

impl BitString {
    fn new(bits: u128, count: u8) -> BitString {
        BitString { bits, count }
    }
}

struct BitIterator {
    bits: u128,
    mask: u128,
}

impl BitIterator {
    fn new(bits: u128, mask: u128) -> Self {
        BitIterator { bits, mask }
    }
}

impl Iterator for BitIterator {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == 0 {
            return None;
        }
        let result = self.bits & self.mask != 0;
        self.mask >>= 1;
        Some(result)
    }
}

impl BitString {
    fn bit_iterator(&self) -> BitIterator {
        if self.count < 1 {
            BitIterator::new(0, 0)
        } else {
            BitIterator::new(self.bits, 1 << (self.count - 1))
        }
    }
}

impl Description for BitString {
    fn bits(&self) -> impl Iterator<Item = bool> {
        self.bit_iterator()
    }
    fn length(&self) -> u64 {
        self.count as u64
    }
}

impl Debug for BitString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for b in self.bits() {
            f.write_str(to_binary_digit(b))?;
        }
        Ok(())
    }
}

fn to_binary_digit(b: bool) -> &'static str {
    if b { "1" } else { "0" }
}

struct Distribution {
    counts: Vec<u64>,
}

impl Distribution {
    fn new(counts: Vec<u64>) -> Distribution {
        Distribution { counts }
    }
}

impl Debug for Distribution {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("[#")?;
        for c in &self.counts {
            f.write_fmt(format_args!(" {c}"))?;
        }
        f.write_str(" #]")
    }
}

struct Pair {
    count: u64,
    index: usize,
}

impl Pair {
    fn new(count: u64, index: usize) -> Self {
        Pair { count, index }
    }
    fn count(&self) -> u64 {
        self.count
    }
}

impl Ord for Pair {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.count < other.count {
            trace!("Pair: {self:?} > {other:?}");
            std::cmp::Ordering::Greater
        } else if self.count == other.count {
            let result = self.index.cmp(&other.index);
            trace!("Pair: {self:?} {result:?} {other:?}");
            result
        } else {
            trace!("Pair: {self:?} < {other:?}");
            std::cmp::Ordering::Less
        }
    }
}

impl PartialOrd for Pair {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Pair {}

impl PartialEq for Pair {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count && self.index == other.index
    }
}

impl Debug for Pair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("<#{},@{}>", self.count, self.index))
    }
}

struct Sorted {
    mapping: BTreeSet<Pair>,
}

pub fn new_distribution(counts: Vec<u64>) -> impl DistributionTrait {
    Distribution { counts }
}

impl Distribution {
    fn encode_private(&self) -> DistributionDescription {
        trace!("Encode(*): [{self:?}]");
        if self.counts.is_empty() {
            return DistributionDescription::Uniform(0);
        }
        let mut sorted = Sorted {
            mapping: BTreeSet::new(),
        };
        for (count, i) in self.counts.iter().zip(1..) {
            sorted.mapping.insert(Pair::new(*count, i));
        }
        let placeholder = vec![Pair::new(0, 0)];
        let it = placeholder
            .iter()
            .chain(sorted.mapping.iter())
            .zip(sorted.mapping.iter())
            .skip(1);
        let mut max = 0;
        let mut threshold = 0;
        for pairs in it {
            let gap = pairs.0.count - pairs.1.count;
            if gap > max {
                max = gap;
                threshold = pairs.0.count();
            }
        }
        if max == 0 {
            return Uniform(self.counts.len());
        }
        let mut probable = vec![];
        let mut improbable = vec![];
        let mut probable_weight = 0;
        let mut improbable_weight = 0;
        let mut bits = 0u128;
        for count in &self.counts {
            bits = bits << 1;
            if *count >= threshold {
                probable.push(*count);
                probable_weight += *count;
                bits |= 1;
            } else {
                improbable.push(*count);
                improbable_weight += *count;
            }
        }
        let split = BitString {
            count: self.counts.len() as u8,
            bits,
        };
        let p = Probability::new(probable_weight, improbable_weight);
        let probable_distribution = Distribution::new(probable);
        let improbable_distribution = Distribution::new(improbable);
        DistributionDescription::Split {
            p,
            split,
            probable: Box::new(probable_distribution.encode_private()),
            improbable: Box::new(improbable_distribution.encode_private()),
        }
    }
}

impl DistributionTrait for Distribution {
    fn encode(&self) -> impl Description {
        let result = self.encode_private();
        let description_length = result.evaluate(self, 1.0).ok();
        info!("Encoded distribution {result:?} (DL: {description_length:?})");
        result
    }
}

#[cfg(test)]
mod test {
    use rand::RngExt;
    use test_log::test;

    use super::*;

    #[test]
    fn test_pair_ordering() {
        let mut rng = rand::rng();
        let mut pairs = vec![];
        for i in 0..5 {
            for j in 0..5 {
                pairs.push(Pair::new(i, j));
            }
        }
        let mut sorted_set = BTreeSet::new();
        while !pairs.is_empty() {
            let i = rng.random_range(0..pairs.len());
            let pair = pairs.swap_remove(i);
            sorted_set.insert(pair);
        }
        let dummy = vec![Pair::new(0, 0)];
        let it = dummy
            .iter()
            .chain(sorted_set.iter())
            .zip(sorted_set.iter())
            .skip(1);
        for (first, second) in it {
            assert!(first.count >= second.count);
            if first.count == second.count {
                assert!(first.index <= second.index)
            }
            info!("Ordered: [{first:?}] before [{second:?}]");
        }
    }

    #[test]
    fn test_prefix_free() {
        info!("Test prefix free encoding of integers");
        for i in 0..18 {
            let pf = PrefixFree(i);
            let bigger = i + 1;
            let bit_length = pf.length();
            info!("Prefix-free: {i} -> {bigger} -> {pf:?} ({bit_length})")
        }
    }
}
