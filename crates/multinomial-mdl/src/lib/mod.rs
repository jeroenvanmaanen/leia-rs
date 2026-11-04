mod implementation;
mod not_supported;

use std::fmt::Debug;

use anyhow::Result;
use rational::Rational;

pub use implementation::new_counts;

use crate::not_supported::NotSupported;

pub trait Description: Debug {
    fn bits(&self) -> impl Iterator<Item = bool>;
    fn length(&self) -> u64;
}

pub trait Counts: Debug {
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> Result<u64>;
    fn iter(&self) -> impl Iterator<Item = u64>;
    fn as_maximum_likelihood(&self) -> Option<&impl MaximumLikelihood> {
        None::<&NotSupported>
    }
    fn as_minimum_description_length(&self) -> Option<&impl MinimumDescriptionLength> {
        None::<&NotSupported>
    }
}

pub trait MultinomialModel: Description {
    fn len(&self) -> usize;
    fn probability(&self, i: usize) -> Result<Rational>;
    fn evaluate<C: Counts>(&self, counts: &C) -> Result<f64>;
    fn length_of_model(&self) -> u64;
    fn length_of_data_given_model<C: Counts>(&self, counts: &C) -> Result<f64>;
}

pub trait MaximumLikelihood: Counts {
    fn maximum_likelihood_model(&self) -> impl MultinomialModel;
}

pub trait MinimumDescriptionLength: Counts {
    fn minimum_description_length_model(&self) -> impl MultinomialModel;
}
