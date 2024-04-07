use std::collections::HashMap;
use std::marker::PhantomData;

use crate::sim::layer::digestion::consumable::Consumable;
use crate::sim::layer::digestion::consumed::Consumed;
use crate::sim::layer::digestion::DigestionDirection;
use crate::sim::{Organism, SimTime};
use crate::substance::substance_wrapper::substance_store_wrapper;
use crate::substance::Substance;
use crate::units::geometry::Volume;
use crate::IdType;

/// Provides methods for Digestion modules to interact with the simulation
pub struct DigestionConnector<O: Organism> {
    pd: PhantomData<O>,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Consumable which is accessible by the current module
    pub(crate) consumed_list: Vec<Consumed>,
    /// Whether all changes should be unscheduled before each run
    /// NOTE: If this is set to false, the component is responsible for
    /// tracking and unscheduling preexisting changes, if necessary
    pub(crate) unschedule_all: bool,
}

impl<O: Organism> DigestionConnector<O> {
    /// Creates a new CoreConnector
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            sim_time: SimTime::from_s(0.0),
            consumed_list: Vec::new(),
            unschedule_all: true,
        }
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Whether to unschedule all changes automatically before each run
    /// NOTE: If this is set to false, the component is responsible for
    /// tracking and unscheduling preexisting changes, if necessary
    pub fn unschedule_all(&mut self, value: bool) {
        self.unschedule_all = value;
    }

    /// Retrieves an iterator of consumables currently owned
    /// by this component
    pub fn consumed(&mut self) -> impl Iterator<Item = &mut Consumed> {
        self.consumed_list.iter_mut()
    }
}
