use std::marker::PhantomData;

use crate::sim::Organism;

pub struct DigestionInitializer<O: Organism + ?Sized> {
    pd: PhantomData<O>
}

impl<O: Organism + ?Sized> DigestionInitializer<O> {
    pub fn new() -> Self {
        Self {
            pd: PhantomData
        }
    }
}
