use std::{
    fmt::Debug,
    iter::{Empty, empty},
};

use anyhow::{Result, anyhow};
use rational::Rational;

use crate::{Counts, Description, MaximumLikelihood, MinimumDescriptionLength, MultinomialModel};

#[derive(Debug)]
pub struct NotSupported {}

impl Description for NotSupported {
    fn bits(&self) -> impl Iterator<Item = bool> {
        empty()
    }

    fn length(&self) -> u64 {
        0
    }
}

impl MultinomialModel for NotSupported {
    fn len(&self) -> usize {
        0
    }

    fn probability(&self, _i: usize) -> Result<Rational> {
        Err(anyhow!("Not supported"))
    }

    fn evaluate<C: Counts>(&self, _counts: &C) -> Result<f64> {
        Err(anyhow!("Not supported"))
    }

    fn length_of_model(&self) -> u64 {
        0
    }

    fn length_of_data_given_model<C: Counts>(&self, _counts: &C) -> Result<f64> {
        Ok(0.0)
    }
}

impl Counts for NotSupported {
    fn len(&self) -> usize {
        0
    }

    fn get(&self, _i: usize) -> Result<u64> {
        Err(anyhow!("Not supported"))
    }

    fn iter(&self) -> impl Iterator<Item = u64> {
        empty::<u64>()
    }
}

impl IntoIterator for &NotSupported {
    type Item = u64;
    type IntoIter = Empty<u64>;

    fn into_iter(self) -> Self::IntoIter {
        empty::<u64>()
    }
}

impl MaximumLikelihood for NotSupported {
    fn maximum_likelihood_model(&self) -> impl MultinomialModel {
        panic!() as NotSupported
    }
}

impl MinimumDescriptionLength for NotSupported {
    fn minimum_description_length_model(&self) -> impl MultinomialModel {
        panic!() as NotSupported
    }
}
