use std::collections::{HashMap, HashSet};
use std::any::TypeId;
use crate::sim::organism::Organism;


pub struct NervousInitializer<O: Organism> {
    /// What type of signals this component should be notified of
    /// and on which nerve sections
    pub(crate) signal_notifies: HashMap<O::NerveType, HashSet<TypeId>>,
}

impl<O: Organism> NervousInitializer<O> {
    pub fn new() -> NervousInitializer<O> {
        NervousInitializer {
            signal_notifies: HashMap::new(),
        }
    }

    pub fn notify_of<T: 'static> (&mut self, nerve: O::NerveType) {
        self.signal_notifies
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>());
    }

}

#[cfg(test)]
pub mod test {
    use super::NervousInitializer;
    
}
