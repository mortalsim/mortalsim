
use std::collections::hash_set;

pub mod core;
pub mod closed_circulation;
pub mod digestion;
pub mod nervous;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum SimLayer {
    Core,
    ClosedCirculation,
    Digestion,
}

pub struct AnatomicalRegionIter<'a, T: Clone>(pub hash_set::Iter<'a, T>);

impl<'a, T: Clone> Iterator for AnatomicalRegionIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        Some(self.0.next()?.clone())
    }
}

pub struct LayerManager {

}
