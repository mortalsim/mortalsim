use std::marker::PhantomData;

use crate::sim::Organism;

pub struct DigestionInitializer<O: Organism> {
    pd: PhantomData<O>
}

impl<O: Organism> DigestionInitializer<O> {
    pub fn new() -> Self {
        Self {
            pd: PhantomData
        }
    }
}
