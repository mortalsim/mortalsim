use std::any::{Any, TypeId};
use std::collections::hash_set;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;
use std::sync::Arc;

use crate::sim::layer::AnatomicalRegionIter;
use crate::sim::organism::Organism;

pub trait Nerve:
    FromStr + Hash + Clone + Copy + Eq + fmt::Debug + fmt::Display + Send + Sync + Into<&'static str>
{
    type AnatomyType: Clone;
    fn terminal_nerves<'a>() -> NerveIter<'a, Self>;
    fn uplink<'a>(&self) -> NerveIter<'a, Self>;
    fn downlink<'a>(&self) -> NerveIter<'a, Self>;
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType>;
}

pub struct NerveIter<'a, N: Nerve>(pub hash_set::Iter<'a, N>);

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

pub struct NerveSignal<O: Organism> {
    path: Vec<O::NerveType>,
    message: Arc<dyn Any>,
}

impl<O: Organism> NerveSignal<O> {
    pub fn new<T: 'static>(neural_path: Vec<O::NerveType>, message: T) -> anyhow::Result<Self> {
        if neural_path.is_empty() {
            return Err(anyhow!("Neural path cannot be empty!"));
        }
        for idx in 0..(neural_path.len()-1) {
            let cur_nerve = neural_path.get(idx).unwrap();
            let next_nerve = neural_path.get(idx+1).unwrap();
            // Ensure each section of the path is valid
            if !cur_nerve.downlink().any(|d| d == *next_nerve) {
                return Err(anyhow!("Invalid link from {} to {}", cur_nerve, next_nerve));
            }
        }

        Ok(Self {
            path: neural_path,
            message: Arc::new(message),
        })
    }

    pub fn neural_path(&self) -> impl Iterator<Item = &O::NerveType> {
        self.path.iter()
    }

    pub fn type_id(&self) -> TypeId {
        self.message.type_id()
    }

    pub fn message<'a, T: 'static>(&'a self) -> &'a T {
        self.message.downcast_ref::<T>().expect("Invalid NerveSignal downcast")
    }
}
