mod implementation;

use std::fmt::Debug;

use anyhow::Result;
use rational::Rational;

pub use implementation::new_counts;

pub trait Description: Debug {
    fn bits(&self) -> impl Iterator<Item = bool>;
    fn length(&self) -> u64;
}

pub trait Counts: Debug {
    fn encode(&self) -> impl MultinomialModel;
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> Result<u64>;
    fn iter(&self) -> impl Iterator<Item = u64>;
}

pub trait MultinomialModel: Description {
    fn len(&self) -> usize;
    fn probability(&self, i: usize) -> Result<Rational>;
    fn evaluate<C: Counts>(&self, counts: &C) -> Result<f64>;
}
