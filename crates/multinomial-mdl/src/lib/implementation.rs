use std::collections::{BTreeSet, VecDeque};
use std::fmt::{Debug, Formatter};
use std::sync::LazyLock;

use anyhow::{Result, anyhow};
use log::{error, info, trace};
use rational::Rational;
use regex::Regex;

use crate::implementation::Decision::{SPLIT, UNIFORM};
use crate::implementation::MultinomialModel::Split;
use crate::implementation::MultinomialModel::Uniform;
use crate::{MaximumLikelihood, MinimumDescriptionLength};

use super::Counts as CountsTrait;
use super::Description;
use super::MultinomialModel as MultinomialModelTrait;

enum MultinomialModel {
    Uniform {
        fraction: f64,
        dimension: usize,
    },
    Split {
        fraction: f64,
        p: Probability,
        split: BitString,
        probable: Box<MultinomialModel>,
        improbable: Box<MultinomialModel>,
    },
}

impl MultinomialModel {
    fn fraction(&self) -> f64 {
        match self {
            Uniform { fraction, .. } => *fraction,
            Split { fraction, .. } => *fraction,
        }
    }
}

impl Description for MultinomialModel {
    fn bits(&self) -> impl Iterator<Item = bool> {
        match self {
            Uniform { dimension, .. } if *dimension < 2 => {
                let empty = vec![];
                bit_vec_bits(&empty)
            }
            Uniform { .. } => {
                let single = vec![BitString::new(0, 1)];
                bit_vec_bits(&single)
            }
            Split { split, .. } => {
                let mut result: Vec<BitString> = vec![];
                result.push(BitString::new(1, 1));
                result.push(split.clone());

                bit_vec_bits(&result);
                todo!("Add bits for probable and improbable")
            }
        }
    }

    fn length(&self) -> u64 {
        match self {
            Uniform { dimension, .. } if *dimension < 2 => 0,
            Uniform { .. } => 1,
            Split {
                split,
                p,
                probable,
                improbable,
                ..
            } => 1 + split.length() + p.length() + probable.length() + improbable.length(),
        }
    }
}

impl MultinomialModelTrait for MultinomialModel {
    fn len(&self) -> usize {
        match self {
            Uniform { dimension, .. } => *dimension as usize,
            Split { split, .. } => split.count as usize,
        }
    }

    fn probability(&self, i: usize) -> Result<Rational> {
        match self {
            Uniform { dimension, .. } => {
                if i != 0 && i <= *dimension {
                    Ok(Rational::new(1, *dimension as u64))
                } else {
                    Err(anyhow!("Invalid index in uniform: {i}"))
                }
            }
            Split {
                p,
                split,
                probable,
                improbable,
                ..
            } => {
                if self.len() <= 1 {
                    return Ok(Rational::one());
                }
                let is_probable = split.bit(i)?;
                let side = if is_probable { probable } else { improbable };
                let side_p = if side.len() > 1 {
                    let mut side_bits = *split;
                    if !is_probable {
                        side_bits.invert();
                    }
                    let mut it = side_bits.bits();
                    let mut k = 1;
                    for _ in 1..i {
                        if let Some(b) = it.next() {
                            if b {
                                k += 1;
                            }
                        } else {
                            return Err(anyhow!("Invalid index in split: {i}"));
                        }
                    }
                    side.probability(k)?
                } else {
                    Rational::one()
                };
                let n = p.numerator.0;
                let w = p.weight.0;
                let nn = if is_probable { n } else { w };
                Ok(Rational::new(nn, n + w) * side_p)
            }
        }
    }

    fn evaluate<C: CountsTrait>(&self, counts: &C) -> Result<f64> {
        let model = self.length_of_model() as f64;
        let data = self.length_of_data_given_model(counts)?;
        Ok(model + data)
    }

    fn length_of_model(&self) -> u64 {
        match self {
            Uniform { dimension, .. } => {
                if *dimension == 0 {
                    return 0;
                } else {
                    return 1;
                }
            }
            Split {
                p,
                split,
                probable,
                improbable,
                ..
            } => {
                let probable_model_length = probable.length_of_model();
                let improbable_model_length = improbable.length_of_model();
                let model_length = 1
                    + p.length()
                    + split.length()
                    + probable_model_length
                    + improbable_model_length;
                model_length
            }
        }
    }

    fn length_of_data_given_model<C: CountsTrait>(&self, counts: &C) -> Result<f64> {
        let fraction = self.fraction();
        let mut result = 0.0;
        for (i, c) in (1..=counts.len()).zip(counts.iter()) {
            let pi = self.probability(i)?.decimal_value() * fraction;
            let item_length = -pi.log2();
            result += c as f64 * item_length;
        }
        Ok(result)
    }
}

impl Debug for MultinomialModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uniform { dimension, .. } if *dimension < 2 => {
                f.debug_tuple("Uniform").field(dimension).finish()?;
                f.write_str(" ''")
            }
            Self::Uniform { dimension, .. } => {
                f.debug_tuple("Uniform").field(dimension).finish()?;
                f.write_str(" '0'")
            }
            Self::Split {
                p,
                split,
                probable,
                improbable,
                ..
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
#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
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
    fn bit(&self, i: usize) -> Result<bool> {
        trace!("Index: {i}");
        if i != 0 && i <= self.count as usize {
            let pos = self.count as usize - i;
            Ok(((1 << pos) & self.bits) != 0)
        } else {
            Err(anyhow!("Invalid index in bit-string: {i}"))
        }
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
    fn invert(&mut self) {
        self.bits ^= u128::MAX;
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

struct Counts {
    counts: Vec<u64>,
}

impl Counts {
    fn new(counts: Vec<u64>) -> Counts {
        Counts { counts }
    }
}

impl Debug for Counts {
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

pub fn new_counts(counts: Vec<u64>) -> impl CountsTrait {
    Counts { counts }
}

impl Counts {
    fn encode_private<S: ModelingStrategy>(&self, strategy: &S, fraction: f64) -> MultinomialModel {
        trace!("Encode(*): [{self:?}]");
        if self.counts.is_empty() {
            return MultinomialModel::Uniform {
                fraction,
                dimension: 0,
            };
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
            return Uniform {
                fraction,
                dimension: self.counts.len(),
            };
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
        let early = strategy.decide_early(probable_weight, improbable_weight);
        let mut p = Probability::new(probable_weight, improbable_weight);
        match &early {
            Some(UNIFORM) => {
                return MultinomialModel::Uniform {
                    fraction,
                    dimension: self.len(),
                };
            }
            Some(SPLIT(probability)) => p = *probability,
            None => (),
        };

        let pn = p.numerator.0;
        let pd = pn + p.weight.0;
        let pr = Rational::new(pn, pd).decimal_value();

        let probable_counts = Counts::new(probable);
        let improbable_counts = Counts::new(improbable);
        let probable_model = probable_counts.encode_private(strategy, fraction * pr);
        let improbable_model = improbable_counts.encode_private(strategy, fraction * (1.0 - pr));
        if early.is_none() {
            p = match strategy
                .decide_late(
                    fraction,
                    &probable_model,
                    &probable_counts,
                    &improbable_model,
                    &improbable_counts,
                )
                .inspect_err(|e| error!("Error in decide_late: [{e:?}]"))
                .unwrap_or(UNIFORM)
            {
                UNIFORM => {
                    return MultinomialModel::Uniform {
                        fraction,
                        dimension: self.len(),
                    };
                }
                SPLIT(p) => p,
            };
        }
        let gcd = num::integer::gcd(p.numerator.0, p.weight.0);
        let p = Probability::new(p.numerator.0 / gcd, p.weight.0 / gcd);
        MultinomialModel::Split {
            fraction,
            p,
            split,
            probable: Box::new(probable_model),
            improbable: Box::new(improbable_model),
        }
    }
}

impl CountsTrait for Counts {
    fn len(&self) -> usize {
        self.counts.len()
    }

    fn get(&self, i: usize) -> Result<u64> {
        self.counts.get(i).map(|c| *c).ok_or(anyhow!(""))
    }

    fn iter(&self) -> impl Iterator<Item = u64> {
        self.counts.iter().map(|c| *c)
    }

    fn as_maximum_likelihood(&self) -> Option<&impl MaximumLikelihood> {
        Some(self)
    }

    fn as_minimum_description_length(&self) -> Option<&impl MinimumDescriptionLength> {
        Some(self)
    }
}

impl MaximumLikelihood for Counts {
    fn maximum_likelihood_model(&self) -> impl MultinomialModelTrait {
        let result = self.encode_private(&MAXIMUM_LIKELIHOOD_STRATEGY, 1.0);
        info!("Encoded distribution {result:?} {{");
        let mut total = Rational::zero();
        for i in 1..=result.len() {
            match result.probability(i) {
                Ok(p) => {
                    total += p;
                    info!("  P[X={i}] = {} / {}", p.numerator(), p.denominator());
                }
                Err(e) => error!("Error: {i}: {e}"),
            }
        }
        info!(
            "}} (total: {} / {})",
            total.numerator(),
            total.denominator()
        );
        result
    }
}

impl MinimumDescriptionLength for Counts {
    fn minimum_description_length_model(&self) -> impl MultinomialModelTrait {
        let result = self.encode_private(&MINIMUM_DESCRIPTION_LENGTH_STRATEGY, 1.0);
        info!("Encoded distribution {result:?} {{");
        let mut total = Rational::zero();
        for i in 1..=result.len() {
            match result.probability(i) {
                Ok(p) => {
                    total += p;
                    info!("  P[X={i}] = {} / {}", p.numerator(), p.denominator());
                }
                Err(e) => error!("Error: {i}: {e}"),
            }
        }
        info!(
            "}} (total: {} / {})",
            total.numerator(),
            total.denominator()
        );
        result
    }
}

enum Decision {
    UNIFORM,
    SPLIT(Probability),
}

trait ModelingStrategy {
    fn decide_early(&self, _probable: u64, _improbable: u64) -> Option<Decision> {
        None
    }
    fn decide_late<M: MultinomialModelTrait, C: CountsTrait>(
        &self,
        _fraction: f64,
        _probable: &M,
        _probable_counts: &C,
        _improbable: &M,
        _improbable_counts: &C,
    ) -> Result<Decision> {
        Ok(UNIFORM)
    }
}

struct MaximumLikelihoodStrategy {}

const MAXIMUM_LIKELIHOOD_STRATEGY: MaximumLikelihoodStrategy = MaximumLikelihoodStrategy {};

impl ModelingStrategy for MaximumLikelihoodStrategy {
    fn decide_early(&self, probable: u64, improbable: u64) -> Option<Decision> {
        Some(SPLIT(Probability::new(probable, improbable)))
    }
}

struct MinimumDescriptionLengthStrategy {}

const MINIMUM_DESCRIPTION_LENGTH_STRATEGY: MinimumDescriptionLengthStrategy =
    MinimumDescriptionLengthStrategy {};

impl ModelingStrategy for MinimumDescriptionLengthStrategy {
    fn decide_late<M: MultinomialModelTrait, C: CountsTrait>(
        &self,
        fraction: f64,
        probable: &M,
        probable_counts: &C,
        improbable: &M,
        improbable_counts: &C,
    ) -> Result<Decision> {
        let mut probable_total = 0;
        let mut uniform_data = vec![];
        for c in probable_counts.iter() {
            probable_total += c;
            uniform_data.push(c);
        }
        let mut improbable_total = 0;
        for c in improbable_counts.iter() {
            improbable_total += c;
            uniform_data.push(c);
        }
        let p = Probability::new(probable_total, improbable_total);
        let probable_length = probable.evaluate(probable_counts)?;
        let improbable_length = improbable.evaluate(improbable_counts)?;
        let uniform_counts = Counts::new(uniform_data);
        let uniform_model = MultinomialModel::Uniform {
            fraction,
            dimension: uniform_counts.len(),
        };
        let uniform_length = uniform_model.evaluate(&uniform_counts)?;
        let split_length = (probable_counts.len() + improbable_counts.len()) as u64;
        if uniform_length
            < (1 + split_length + p.length()) as f64 + probable_length + improbable_length
        {
            Ok(UNIFORM)
        } else {
            Ok(SPLIT(p))
        }
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
