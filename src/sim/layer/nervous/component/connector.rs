use std::any::{Any, TypeId};
use std::collections::{BTreeMap, HashMap};

use downcast_rs::Downcast;

use crate::event::Event;
use crate::sim::layer::nervous::nerve::NerveSignal;
use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::util::{IdType, OrderedTime};

pub trait NerveSignalTransformer: Send {
    fn transform(&mut self, message: &'_ mut dyn Event) -> Option<()>;
}

struct TransformFn<'a, T>(Box<dyn FnMut(&'_ mut T) -> Option<()> + Send + 'a>);

impl<'a, T: Event> NerveSignalTransformer for TransformFn<'a, T> {
    fn transform(&mut self, message: &mut dyn Event) -> Option<()> {
        self.0(message.downcast_mut::<T>().unwrap())
    }
}

pub struct NervousConnector<O: Organism> {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Map of pending notifications. Note this is included here because
    /// only the component has type context to accurately execute transformations
    /// on previously scheduled signals.
    pub(crate) pending_signals: BTreeMap<OrderedTime, Vec<NerveSignal<O>>>,
    /// Signal transformers on given nerve segments
    pub(crate) transforms:
        HashMap<O::NerveType, HashMap<TypeId, HashMap<IdType, Box<dyn NerveSignalTransformer>>>>,
    /// Incoming signals
    pub(crate) incoming: HashMap<TypeId, Vec<NerveSignal<O>>>,
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
            outgoing: Vec::new(),
            transforms: HashMap::new(),
            adding_transforms: HashMap::new(),
            registered_transforms: HashMap::new(),
            removing_transforms: HashMap::new(),
            pending_signals: BTreeMap::new(),
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
