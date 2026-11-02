mod implementation;

use std::fmt::Debug;

use anyhow::Result;
use rational::Rational;

pub use implementation::new_distribution;

pub trait Description: Debug {
    fn bits(&self) -> impl Iterator<Item = bool>;
    fn length(&self) -> u64;
}

pub trait Counts {
    fn encode(&self) -> impl MultinomialModel;
}

pub trait MultinomialModel: Description {
    fn len(&self) -> usize;
    fn probability(&self, i: usize) -> Result<Rational>;
}
