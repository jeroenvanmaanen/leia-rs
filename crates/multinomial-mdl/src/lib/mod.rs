mod implementation;

use std::fmt::Debug;

pub use implementation::new_distribution;

pub trait Description: Debug {
    fn bits(&self) -> impl Iterator<Item = bool>;
    fn length(&self) -> u64;
}

pub trait Distribution {
    fn encode(&self) -> impl Description;
}
