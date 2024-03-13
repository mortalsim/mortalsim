use std::any::{Any, TypeId};
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::{Mutex, OnceLock};

use crate::event::Event;
use crate::sim::layer::AnatomicalRegionIter;
use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::util::IdGenerator;
use crate::IdType;


pub trait Nerve:
    FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Into<&'static str>
{
    type AnatomyType: Clone;
    fn terminal_nerves<'a>() -> NerveIter<'a, Self>;
    fn uplink<'a>(&self) -> NerveIter<'a, Self>;
    fn downlink<'a>(&self) -> NerveIter<'a, Self>;
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType>;
}

pub struct NerveIter<'a, N: Nerve>(pub core::slice::Iter<'a, N>);

impl<'a, N: Nerve> Iterator for NerveIter<'a, N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        Some(self.0.next()?.clone())
    }
}

impl<'a, N: Nerve> ExactSizeIterator for NerveIter<'a, N> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
