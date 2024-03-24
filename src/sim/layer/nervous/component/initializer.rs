use crate::event::Event;
use crate::sim::layer::nervous::transform::{NerveSignalTransformer, TransformFn};
use crate::sim::organism::Organism;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

pub struct NervousInitializer<O: Organism> {
    /// What type of signals this component should be notified of
    /// and on which nerve sections
    pub(crate) signal_notifies: HashMap<O::NerveType, HashSet<TypeId>>,
    /// Transformations to add
    pub(crate) adding_transforms:
        HashMap<O::NerveType, HashMap<TypeId, Box<dyn NerveSignalTransformer>>>,
}

impl<O: Organism> NervousInitializer<O> {
    pub fn new() -> NervousInitializer<O> {
        NervousInitializer {
            signal_notifies: HashMap::new(),
            adding_transforms: HashMap::new(),
        }
    }

    pub fn notify_of<T: 'static>(&mut self, nerve: O::NerveType) {
        self.signal_notifies
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>());
    }

    pub fn transform_message<T: Event>(
        &mut self,
        nerve: O::NerveType,
        handler: impl (FnMut(&mut T) -> Option<&mut T>) + Send + 'static,
    ) {

        self.adding_transforms
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>(), Box::new(TransformFn(Box::new(handler))));
    }

}

#[cfg(test)]
pub mod test {}
