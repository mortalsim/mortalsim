use std::collections::{HashMap, BTreeMap};
use std::any::{TypeId, Any};

use downcast_rs::Downcast;

use crate::event::Event;
use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::sim::layer::nervous::nerve::NerveSignal;
use crate::util::{IdType, OrderedTime};

struct TransformFn<'a, T>(Box<dyn FnMut(&mut T) -> Option<()> + 'a>);

impl<'a, T> TransformFn<'a, T> {
    pub fn transform(&mut self, message: &mut T) -> Option<()> {
        self.0(message)
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
    pub(crate) transforms: HashMap<O::NerveType, HashMap<TypeId, HashMap<IdType, Box<dyn Any>>>>,
    /// Incoming signals
    pub(crate) incoming: HashMap<TypeId, Vec<NerveSignal<O>>>,
    /// Outgoing signals
    pub(crate) outgoing: Vec<NerveSignal<O>>,
    /// Transformations to add
    pub(crate) adding_transforms: HashMap<O::NerveType, HashMap<TypeId, Box<dyn Any>>>,
    /// Map of registered transformations
    pub(crate) registered_transforms: HashMap<O::NerveType, HashMap<TypeId, IdType>>,
    /// Map of removing transformations
    pub(crate) removing_transforms: HashMap<O::NerveType, HashMap<TypeId, IdType>>,
    /// Empty Event list for ergonomic message use
    empty: Vec<NerveSignal<O>>
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

    pub fn get_messages<T: Event>(&self) -> impl Iterator<Item=&'_ T> {
        match self.incoming.get(&TypeId::of::<T>()) {
            Some(signals) => {
                either::Left(signals.iter().map(|s| s.message::<T>()))
            }
            None => {
                either::Right(self.empty.iter().map(|s| s.message::<T>()))
            }
        }
        //             .downcast_arc()
        //             .expect("Invalid message type retrieval");
        // Some(evt)
    }

    pub fn send_message<T: Event>(
        &mut self,
        mut message: T,
        neural_path: Vec<O::NerveType>,
        send_time: SimTime,
    ) -> anyhow::Result<()> {
        if send_time <= self.sim_time {
            return Err(anyhow!("Invalid send_time: time must be greater than the current time!"))
        }

        let mut block = false;

        // See if we need to transform the outgoing signal
        for nerve in neural_path.iter() {
            if let Some(fn_map) = self.transforms.get_mut(nerve) {
                if let Some(transform_list) = fn_map.get_mut(&TypeId::of::<T>()) {
                    for (_, transform_box) in transform_list.iter_mut() {
                        let t = transform_box.as_any_mut().downcast_mut::<TransformFn<T>>().unwrap();
                        if None == t.transform(&mut message) {
                            block = true;
                        }
                    }
                }
            }
        }

        let mut signal = NerveSignal::new(message, neural_path, send_time)?;
        if block {
            signal.block();
        }

        self.outgoing.push(signal);
        Ok(())
    }

    pub fn transform_message<T: Event>(
        &mut self,
        nerve: O::NerveType,
        mut handler: impl (FnMut(&mut T) -> Option<()>) + 'static,
    ) {
        let type_id = TypeId::of::<T>();
        // Execute on any pending notifications to see if they need to change
        for (_, signals) in self.pending_signals.iter_mut() {
            for mut signal in signals.drain(..).filter(|s| s.type_id() == type_id) {
                if None == handler(signal.message_mut::<T>()) {
                    signal.block();
                }
            }
        }

        self.adding_transforms
            .entry(nerve)
            .or_default()
            .insert(TypeId::of::<T>(), Box::new(TransformFn(Box::new(handler))));
    }

    pub fn stop_transform<T: 'static>(
        &mut self,
        nerve: O::NerveType,
    ) -> anyhow::Result<()> {
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
pub mod test {


}
