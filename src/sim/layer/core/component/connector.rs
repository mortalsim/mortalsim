use crate::event::Event;
use crate::sim::{Organism, SimState, SimTime};
use crate::util::id_gen::IdType;
use anyhow::Result;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

/// Provides methods for `Sim` modules to interact with the simulation
pub struct CoreConnector<O: Organism> {
    pd: PhantomData<O>,
    /// State specific to the connected module
    pub(crate) sim_state: SimState,
    /// Holds a shared reference to the Event which triggered module execution
    pub(crate) trigger_events: Vec<TypeId>,
    /// Map of scheduled event identifiers
    pub(crate) scheduled_events: HashMap<TypeId, HashMap<IdType, SimTime>>,
    /// Map of scheduled ids to event types
    pub(crate) schedule_id_type_map: HashMap<IdType, TypeId>,
    /// List of events to schedule
    pub(crate) pending_schedules: Vec<(SimTime, Box<dyn Event>)>,
    /// List of events to unschedule
    pub(crate) pending_unschedules: Vec<IdType>,
    /// Copy of the current simulation time
    pub(crate) sim_time: SimTime,
    /// Whether to indicate to the parent Sim that all previously scheduled events should be unscheduled
    pub(crate) unschedule_all: bool,
}

impl<O: Organism> CoreConnector<O> {
    /// Creates a new CoreConnector
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            // Temporary empty state which will be replaced by the canonical state
            sim_state: SimState::new(),
            trigger_events: Vec::new(),
            scheduled_events: HashMap::new(),
            schedule_id_type_map: HashMap::new(),
            pending_schedules: Vec::new(),
            pending_unschedules: Vec::new(),
            sim_time: SimTime::from_s(0.0),
            unschedule_all: true,
        }
    }

    /// Schedules an `Event` for future emission after a specified delay
    ///
    /// ### Arguments
    /// * `wait_time` - Amount of time to wait before execution
    /// * `evt` - `Event` to emit after `wait_time` has elapsed
    pub fn schedule_event(&mut self, wait_time: SimTime, evt: impl Event) {
        self.pending_schedules.push((wait_time, Box::new(evt)))
    }

    /// Whether to unschedule all previously scheduled `Event` objects (default is true)
    /// Set to `false` in order to manually specify which `Event` objects to unschedule
    /// using `unschedule_event`
    pub fn unschedule_all(&mut self, setting: bool) {
        self.unschedule_all = setting;
    }

    /// Unschedules an `Event` which has been scheduled previously.
    ///
    /// ### Arguments
    /// * `schedule_id` - schedule id of the Event to unschedule
    ///
    /// Returns Ok if the id is valid, and Err otherwise
    pub fn unschedule_event<E: Event>(&mut self, schedule_id: IdType) -> Result<()> {
        let type_id = TypeId::of::<E>();
        match self.scheduled_events.get(&type_id) {
            Some(smap) => {
                if smap.contains_key(&schedule_id) {
                    Ok(())
                } else {
                    Err(anyhow!("Invalid id provided for unscheduling"))
                }
            }
            None => Err(anyhow!("Invalid type provided for unscheduling")),
        }
    }

    /// Retrieves a mapping of schedule ids -> execution time for each
    /// instance of the given Event type which has been scheduled previously,
    /// and which has not yet been emitted.
    ///
    /// Returns a HashMap if any events are scheduled for the given type, and
    /// None otherwise
    pub fn get_scheduled_events<'a, E: Event>(&'a mut self) -> impl Iterator<Item = (&'a IdType, &'a SimTime)> {
        self.scheduled_events.entry(TypeId::of::<E>()).or_default().iter()
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Retrieves the current `Event` object from state as an Arc
    pub fn get<E: Event>(&self) -> Option<Box<E>> {
        match self
            .sim_state.get_state_ref(&TypeId::of::<E>())?.downcast::<E>()
        {
            Err(_) => None,
            Ok(typed_evt_rc) => Some(typed_evt_rc),
        }
    }
    
    /// Retrieves the `Event` object(s) which triggered the current `run` (if any)
    pub fn trigger_events<'a>(&'a self) -> impl Iterator<Item = &TypeId> + 'a {
        self.trigger_events.iter()
    }
}

#[cfg(test)]
pub mod test {
    use std::any::TypeId;
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::event::test::TestEventB;
    use crate::sim::test::TestSim;
    use crate::sim::SimState;
    use crate::event::test::TestEventA;
    use crate::units::base::Amount;
    use crate::units::base::Distance;
    use crate::units::base::Time;

    use super::CoreConnector;

    fn basic_event_a() -> TestEventA {
        TestEventA::new(Distance::from_m(1.0))
    }

    fn basic_event_b() -> TestEventB {
        TestEventB::new(Amount::from_mol(1.0))
    }

    fn connector() -> CoreConnector<TestSim> {
        let mut connector = CoreConnector::new();
        let mut a_events = HashMap::new();
        a_events.insert(1, Time::from_s(1.0));
        let mut b_events = HashMap::new();
        b_events.insert(2, Time::from_s(2.0));
        connector.scheduled_events.insert(TypeId::of::<TestEventA>(), a_events);
        connector.scheduled_events.insert(TypeId::of::<TestEventB>(), b_events);
        connector.sim_state = SimState::new();

        let evt_a = Box::new(basic_event_a());
        connector.sim_state.put_state(evt_a.clone());
        connector.trigger_events.push(TypeId::of::<TestEventA>());
        connector.sim_time = Time::from_s(0.0);
        connector
    }
    
    fn connector_with_a_only() -> CoreConnector<TestSim> {
        let mut connector = CoreConnector::new();
        let mut a_events = HashMap::new();
        a_events.insert(1, Time::from_s(1.0));
        connector.scheduled_events.insert(TypeId::of::<TestEventA>(), a_events);
        connector
    }


    #[test]
    pub fn test_emit() {
        let mut connector = CoreConnector::<TestSim>::new();
        connector.schedule_event(Time::from_s(1.0), basic_event_a())
    }
    
    #[test]
    pub fn test_unschedule() {
        let mut connector = connector();
        assert!(connector.unschedule_event::<TestEventA>(1).is_ok());
        assert!(connector.unschedule_event::<TestEventB>(2).is_ok());
    }
    
    #[test]
    pub fn test_unschedule_invalid_event() {
        let mut connector = connector_with_a_only();
        assert!(connector.unschedule_event::<TestEventB>(2).is_err());
    }
    
    #[test]
    pub fn test_unschedule_invalid_id() {
        let mut connector = connector_with_a_only();
        assert!(connector.unschedule_event::<TestEventA>(2).is_err());
    }
    
    #[test]
    pub fn test_unschedule_all() {
        let mut connector = CoreConnector::<TestSim>::new();
        connector.unschedule_all(true);
        assert!(connector.unschedule_all == true);
    }
    
    #[test]
    pub fn test_get_scheduled() {
        let mut connector = connector();

        for (schedule_id, time) in connector.get_scheduled_events::<TestEventA>() {
            assert!(schedule_id == &1);
            assert!(time == &Time::from_s(1.0))
        }
        for (schedule_id, time) in connector.get_scheduled_events::<TestEventB>() {
            assert!(schedule_id == &2);
            assert!(time == &Time::from_s(2.0))
        }
    }
    
    #[test]
    pub fn test_get_time() {
        let connector = connector();
        assert!(connector.sim_time() == Time::from_s(0.0));
    }
    
    #[test]
    pub fn test_get() {
        let connector = connector();
        assert!(connector.get::<TestEventA>().unwrap().as_ref().len == basic_event_a().len);
        assert!(connector.get::<TestEventB>().is_none());
    }
    
    #[test]
    pub fn test_trigger() {
        let connector = connector();
        let mut count = 0;
        let v: Vec<&TypeId> = connector.trigger_events().inspect(|_| {count += 1}).collect();
        assert!(count == 1);
        assert!(v.get(0).unwrap() == &&TypeId::of::<TestEventA>())
    }
}
