use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use downcast_rs::Downcast;

use crate::event::Event;
use crate::sim::layer::nervous::nerve::NerveSignal;
use crate::sim::layer::nervous::transform::{TransformFn, NerveSignalTransformer};
use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::util::{IdType, OrderedTime};

pub struct NervousConnector<O: Organism> {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Incoming signals
    pub(crate) incoming: HashMap<TypeId, Vec<NerveSignal<O>>>,
    /// Incoming signals (thread safe)
    pub(crate) incoming_sync: HashMap<TypeId, Vec<Arc<Mutex<NerveSignal<O>>>>>,
    /// Outgoing signals
    pub(crate) outgoing: Vec<NerveSignal<O>>,
    /// Transformations to add
    pub(crate) adding_transforms:
        HashMap<O::NerveType, HashMap<TypeId, Box<dyn NerveSignalTransformer>>>,
    /// Map of registered transformations
    pub(crate) registered_transforms: HashMap<O::NerveType, HashMap<TypeId, IdType>>,
    /// Map of removing transformations
    pub(crate) removing_transforms: HashMap<O::NerveType, HashMap<TypeId, IdType>>,
    /// Empty Event list for ergonomic message use
    empty: Vec<NerveSignal<O>>,
}

impl<O: Organism + 'static> NervousConnector<O> {
    pub fn new() -> Self {
        Self {
            sim_time: SimTime::from_s(0.0),
            incoming: HashMap::new(),
            incoming_sync: HashMap::new(),
            outgoing: Vec::new(),
            adding_transforms: HashMap::new(),
            registered_transforms: HashMap::new(),
            removing_transforms: HashMap::new(),
            empty: Vec::new(),
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    pub fn get_messages<T: Event>(&self) -> impl Iterator<Item = &'_ T> {
        match self.incoming.get(&TypeId::of::<T>()) {
            Some(signals) => either::Left(signals.iter().map(|s| s.message::<T>())),
            None => either::Right(self.empty.iter().map(|s| s.message::<T>())),
        }
    }

    pub fn send_message<T: Event>(
        &mut self,
        message: T,
        neural_path: Vec<O::NerveType>,
        send_time: SimTime,
    ) -> anyhow::Result<()> {
        if send_time <= self.sim_time {
            return Err(anyhow!(
                "Invalid send_time: time must be greater than the current time!"
            ));
        }

        let signal = NerveSignal::new(message, neural_path, send_time)?;

        self.outgoing.push(signal);
        Ok(())
    }

    pub fn transform_message<T: Event>(
        &mut self,
        nerve: O::NerveType,
        handler: impl (FnMut(&mut T) -> Option<()>) + Send + 'static,
    ) {

        self.adding_transforms
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>(), Box::new(TransformFn(Box::new(handler))));
    }

    pub fn stop_transform<T: 'static>(&mut self, nerve: O::NerveType) -> anyhow::Result<()> {
        if let Some(type_map) = self.registered_transforms.get(&nerve) {
            if type_map.contains_key(&TypeId::of::<T>()) {
                let type_map = self.registered_transforms.remove(&nerve).unwrap();
                self.removing_transforms.insert(nerve, type_map);
                return Ok(());
            }
        }
        Err(anyhow!("Transformation not registered for {}", nerve))
    }
}

#[cfg(test)]
pub mod test {}
