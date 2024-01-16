use std::collections::HashMap;
use std::any::{TypeId, Any};
use std::iter::Map;

use crate::sim::organism::Organism;
use crate::sim::SimTime;
use crate::sim::layer::nervous::nerve::NerveSignal;


pub struct NervousConnector<O: Organism> {
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Incoming signals
    pub(crate) incoming: HashMap<TypeId, NerveSignal<O>>,
    /// Outgoing signals
    pub(crate) outgoing: HashMap<TypeId, Vec<NerveSignal<O>>>,
}

impl<O: Organism> NervousConnector<O> {
    pub fn new() -> Self {
        Self {
            sim_time: SimTime::from_s(0.0),
            incoming: HashMap::new(),
            outgoing: HashMap::new(),
        }
    }

    pub fn get_message<'a, T: 'static>(&'a self) -> Option<&'a T> {
        Some(self.incoming.get(&TypeId::of::<T>())?.message())
    }

    pub fn send_message<T: 'static>(
        &mut self,
        neural_path: Vec<O::NerveType>,
        message: T
    ) -> anyhow::Result<()> {
        let signal = NerveSignal::new(neural_path, message)?;
        self.outgoing.entry(TypeId::of::<T>()).or_default().push(signal);
        Ok(())
    }
}

#[cfg(test)]
pub mod test {


}
