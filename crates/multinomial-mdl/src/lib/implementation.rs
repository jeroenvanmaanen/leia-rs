use std::collections::{BTreeSet, VecDeque};
use std::fmt::{Debug, Formatter};

use log::{info, trace};

use crate::implementation::DistributionDescription::Split;
use crate::implementation::DistributionDescription::Uniform;

use super::Description;
use super::Distribution as DistributionTrait;

#[derive(Debug)]
enum DistributionDescription {
    Uniform(usize),
    Split {
        p: Probability,
        split: BitString,
        probable: Box<DistributionDescription>,
        improbable: Box<DistributionDescription>,
    },
}

impl Description for DistributionDescription {
    fn bits(&self) -> impl Iterator<Item = bool> {
        match self {
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

/// p = n / (n + w) \in [0, 1]
struct Probability {
    numerator: u64,
    weight: u64,
}

impl Probability {
    fn new(numerator: u64, weight: u64) -> Probability {
        Probability { numerator, weight }
    }
}

impl Debug for Probability {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = format!("({}/{})", self.numerator, self.numerator + self.weight);
        f.write_str(&s)
    }
}

impl Description for u64 {
    fn bits(&self) -> impl Iterator<Item = bool> {
        let v = to_bit_strings(*self);
        bit_vec_bits(&v)
    }
    fn length(&self) -> u64 {
        let bit_strings = to_bit_strings(*self);
        bit_strings.length()
    }
}

fn to_bit_strings(n: u64) -> Vec<BitString> {
    let bigger = n + 1;
    let length = bigger.ilog2();
    let mut bit_strings = vec![BitString {
        bits: bigger as u128,
        count: (length + 1) as u8,
    }];
    while length > 0 {
        let length_of_length = length.ilog2();
        let leading_bit = 1 << length_of_length;
        let length_string = BitString {
            bits: (length ^ leading_bit) as u128,
            count: (length_of_length + 1) as u8,
        };
        bit_strings.insert(0, length_string);
    }
    bit_strings
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
            if b {
                f.write_str("1")?;
            } else {
                f.write_str("0")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Distribution {
    counts: Vec<u64>,
}

impl Distribution {
    fn new(counts: Vec<u64>) -> Distribution {
        Distribution { counts }
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
        let formatted = format!("<#{},@{}>", self.count, self.index);
        f.write_str(&formatted)
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
        info!("Encoded distribution {result:?}");
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
}
