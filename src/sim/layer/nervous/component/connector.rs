use std::collections::{HashMap, HashSet};
use std::any::{TypeId, Any};
use std::iter::Map;

use downcast_rs::Downcast;

use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::sim::layer::nervous::nerve::NerveSignal;
use crate::util::IdType;

struct TransformFn<T>(Box<dyn FnMut(T) -> Option<T>>);

impl<T> TransformFn<T> {
    pub fn transform(&mut self, message: T) -> Option<T> {
        self.0(message)
    }
}

pub struct NervousConnector<O: Organism> {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Incoming signals
    pub(crate) incoming: HashMap<TypeId, NerveSignal<O>>,
    /// Outgoing signals
    pub(crate) outgoing: HashMap<TypeId, Vec<NerveSignal<O>>>,
    /// Signal transformers on given nerve segments
    pub(crate) transforming: HashMap<O::NerveType, HashMap<TypeId, Box<dyn Any>>>,
    /// Transformations to add
    pub(crate) adding_transforms: HashMap<O::NerveType, HashMap<TypeId, Box<dyn Any>>>,
    /// Map of registered transformations
    pub(crate) registered_transforms: HashMap<O::NerveType, HashSet<TypeId>>,
    /// Map of removing transformations
    pub(crate) removing_transforms: HashMap<O::NerveType, HashSet<TypeId>>,
}

impl<O: Organism> NervousConnector<O> {
    pub fn new() -> Self {
        Self {
            sim_time: SimTime::from_s(0.0),
            incoming: HashMap::new(),
            outgoing: HashMap::new(),
            transforming: HashMap::new(),
            adding_transforms: HashMap::new(),
            registered_transforms: HashMap::new(),
            removing_transforms: HashMap::new(),
        }
    }

    pub fn get_message<'a, T: 'static>(&'a self) -> Option<&'a T> {
        Some(self.incoming.get(&TypeId::of::<T>())?.message())
    }

    pub fn send_message<T: 'static>(
        &mut self,
        mut message: T,
        neural_path: Vec<O::NerveType>,
        send_time: SimTime,
    ) -> anyhow::Result<()> {
        if send_time < self.sim_time {
            return Err(anyhow!("Invalid send_time: time {} has already passed!", send_time))
        }

        // See if we need to transform the outgoing signal
        for nerve in neural_path.iter() {
            if let Some(fn_map) = self.transforming.get_mut(nerve) {
                if let Some(transform_box) = fn_map.get_mut(&TypeId::of::<T>()) {
                    let t = transform_box.as_any_mut().downcast_mut::<TransformFn<T>>().unwrap();
                    match t.transform(message) {
                        None => {
                            // If the transformer cancels the signal,
                            // we can just return here without adding it.
                            return Ok(());
                        }
                        Some(msg) => {
                            message = msg;
                        }
                    }
                }
            }
        }

        let signal = NerveSignal::new(message, neural_path, send_time)?;
        self.outgoing.entry(TypeId::of::<T>()).or_default().push(signal);
        Ok(())
    }

    pub fn transform_message<'a, T: 'static>(
        &'a mut self,
        nerve: O::NerveType,
        handler: impl FnMut(T) -> Option<T>,
    ) {
        self.adding_transforms
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>(), Box::new(TransformFn(Box::new(handler))));
    }

    pub fn stop_transform<'a, T: 'static>(
        &'a mut self,
        nerve: O::NerveType,
    ) -> anyhow::Result<()> {
        if let Some(types) = self.registered_transforms.get(&nerve) {
            if types.contains(&TypeId::of::<T>()) {
                return Ok(());
            }
        }
        Err(anyhow!("Transformation not registered for {}", nerve))
    }
}

#[cfg(test)]
pub mod test {


}
