use crate::event::Event;
use crate::hub::event_transformer::TransformerItem;
use crate::hub::EventTransformer;
use crate::sim::{Organism, SimState, SimTime};
use crate::id_gen::IdType;
use crate::IdGenerator;
use anyhow::Result;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

/// Provides methods for `Core` modules to interact with the simulation
pub struct CoreConnector<O: Organism> {
    pd: PhantomData<O>,
    /// Local id generator for transformation registration
    pub(crate) id_gen: IdGenerator,
    /// Historical State specific to the connected module
    pub(crate) sim_state: SimState,
    /// events that are actively being emitted this cycle
    pub(crate) active_events: Vec<Arc<dyn Event>>,
    /// Holds a list of Event types which triggered module execution, if applicable
    pub(crate) trigger_events: Vec<TypeId>,
    /// Map of local ids to layer schedule ids
    pub(crate) scheduled_id_map: HashMap<IdType, IdType>,
    /// Map of local ids to layer transform ids
    pub(crate) transform_id_map: HashMap<IdType, IdType>,
    /// List of events to schedule
    pub(crate) pending_schedules: Vec<(SimTime, (IdType, Box<dyn Event>))>,
    /// List of events to unschedule
    pub(crate) pending_unschedules: Vec<IdType>,
    /// Transforms pending from the last run of the component
    pub(crate) pending_transforms: Vec<(IdType, Box<dyn EventTransformer>)>,
    /// List of transforms to unschedule
    pub(crate) pending_untransforms: Vec<IdType>,
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
            id_gen: IdGenerator::new(),
            // Temporary empty state which will be replaced by the canonical state
            sim_state: SimState::new(),
            active_events: Vec::new(),
            trigger_events: Vec::new(),
            scheduled_id_map: HashMap::new(),
            transform_id_map: HashMap::new(),
            pending_schedules: Vec::new(),
            pending_unschedules: Vec::new(),
            pending_transforms: Vec::new(),
            pending_untransforms: Vec::new(),
            sim_time: SimTime::from_s(0.0),
            unschedule_all: true,
        }
    }

    /// Schedules an `Event` for future emission after a specified delay
    ///
    /// ### Arguments
    /// * `wait_time` - Amount of time to wait before execution
    /// * `evt` - `Event` to emit after `wait_time` has elapsed
    pub fn schedule_event(&mut self, wait_time: SimTime, evt: impl Event) -> IdType {
        let schedule_id = self.id_gen.get_id();
        self.pending_schedules.push((wait_time, (schedule_id, Box::new(evt))));
        schedule_id
    }

    /// Whether to unschedule all previously scheduled `Event` objects (default is true)
    /// Set to `false` in order to manually specify which `Event` objects to unschedule
    /// using `unschedule_event`
    /// 
    /// ### Arguments
    /// * `setting` - whether to turn automatic unscheduling on or off
    pub fn unschedule_all(&mut self, setting: bool) {
        self.unschedule_all = setting;
    }

    /// Unschedules an `Event` which has been scheduled previously.
    ///
    /// ### Arguments
    /// * `schedule_id` - schedule id of the Event to unschedule
    ///
    /// Returns Ok if the id is valid, and Err otherwise
    pub fn unschedule_event(&mut self, schedule_id: IdType) -> Result<()> {
        if let Some(layer_schedule_id) = self.scheduled_id_map.remove(&schedule_id) {
            self.pending_unschedules.push(layer_schedule_id);
            return Ok(())
        }
        Err(anyhow!("Invalid schedule_id provided"))
    }

    /// Retrieves the current simulation time
    pub fn sim_time(&self) -> SimTime {
        self.sim_time
    }

    /// Retrieves a reference to the current `Event` object from state
    /// or from active events
    pub fn get<E: Event>(&self) -> Option<&E> {
        if let Some(evt) = self.sim_state.get_state::<E>() {
            return Some(evt);
        }
        // Search in reverse order so the most recent event
        // takes precedence
        for evt in self.active_events.iter().rev() {
            if evt.is::<E>() {
                return Some(evt.downcast_ref::<E>().unwrap());
            }
        }
        
        None
    }

    /// Retrieves any active events of the given type
    pub fn get_active<E: Event>(&self) -> impl Iterator<Item = &E> {
        self.active_events.iter()
            .filter(|evt| evt.is::<E>())
            .map(|evt| evt.downcast_ref::<E>().unwrap())
    }

    /// Retrieves the `Event` object(s) which triggered the current `run` (if any)
    pub fn trigger_events<'a>(&'a self) -> impl Iterator<Item = &TypeId> + 'a {
        self.trigger_events.iter()
    }

    fn register_transform<E: Event>(&mut self, transformer: TransformerItem<'static, E>) -> IdType {
        let local_id = self.id_gen.get_id();

        self.pending_transforms
            .push((local_id, Box::new(transformer)));

        local_id
    }

    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim`.
    ///
    /// ### Arguments
    /// * `handler` - Function to modify the `Event`
    /// 
    /// Returns a registration id for this transformer
    pub fn transform<E: Event>(&mut self, handler: impl FnMut(&mut E) + Send + 'static) -> IdType {
        self.register_transform(TransformerItem::new(handler))
    }

    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim` with a given priority value.
    ///
    /// ### Arguments
    /// * `priority` - Transformation order priority for this registration
    /// * `handler` - Function to modify the `Event`
    /// 
    /// Returns a registration id for this transformer
    pub fn transform_prioritized<E: Event>(
        &mut self,
        priority: i32,
        handler: impl FnMut(&mut E) + Send + 'static,
    ) -> IdType {
        self.register_transform(TransformerItem::new_prioritized(
            handler, priority,
        ))
    }

    /// Unregisters a previously set transformation function.
    ///
    /// ### Arguments
    /// * `transform_id` - Id returned from a previous `transform` or `transform_prioritized` call
    /// 
    /// Returns Ok if successful, and Err if the transform_id is invalid
    pub fn unset_transform(&mut self, transform_id: &IdType) -> anyhow::Result<()> {
        if let Some(layer_transform_id) = self.transform_id_map.remove(transform_id) {
            return Ok(self.pending_untransforms.push(layer_transform_id));
        }
        Err(anyhow!("Invalid transform_id provided"))
    }
}

#[cfg(test)]
pub mod test {
    use std::any::TypeId;
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::event::test::TestEventA;
    use crate::event::test::TestEventB;
    use crate::sim::organism::test::{TestOrganism, TestSim};
    use crate::sim::SimState;
    use crate::units::base::Amount;
    use crate::units::base::Distance;
    use crate::units::base::Time;
    use crate::SimTime;

    use super::CoreConnector;

    pub fn basic_event_a() -> TestEventA {
        TestEventA::new(Distance::from_m(1.0))
    }

    pub fn basic_event_b() -> TestEventB {
        TestEventB::new(Amount::from_mol(1.0))
    }

    fn connector() -> CoreConnector<TestOrganism> {
        let mut connector = CoreConnector::new();
        connector.scheduled_id_map.insert(1, 1);
        connector.scheduled_id_map.insert(2, 2);
        connector.sim_state = SimState::new();

        let evt_a = Arc::new(basic_event_a());
        connector.sim_state.put_state(evt_a.clone());
        connector.trigger_events.push(TypeId::of::<TestEventA>());
        connector.sim_time = SimTime::from_s(0.0);
        connector
    }

    fn connector_with_a_only() -> CoreConnector<TestOrganism> {
        let mut connector = CoreConnector::new();
        connector.scheduled_id_map.insert(1,1);
        connector
    }

    #[test]
    pub fn test_emit() {
        let mut connector = CoreConnector::<TestOrganism>::new();
        connector.schedule_event(SimTime::from_s(1.0), basic_event_a());
    }

    #[test]
    pub fn test_unschedule() {
        let mut connector = connector();
        assert!(connector.unschedule_event(1).is_ok());
        assert!(connector.unschedule_event(2).is_ok());
    }

    #[test]
    pub fn test_unschedule_invalid_event() {
        let mut connector = connector_with_a_only();
        assert!(connector.unschedule_event(2).is_err());
    }

    #[test]
    pub fn test_unschedule_invalid_id() {
        let mut connector = connector_with_a_only();
        assert!(connector.unschedule_event(2).is_err());
    }

    #[test]
    pub fn test_unschedule_all() {
        let mut connector = CoreConnector::<TestOrganism>::new();
        connector.unschedule_all(true);
        assert!(connector.unschedule_all == true);
    }

    #[test]
    pub fn test_get_time() {
        let connector = connector();
        assert!(connector.sim_time() == SimTime::from_s(0.0));
    }

    #[test]
    pub fn test_get() {
        let connector = connector();
        assert!(connector.get::<TestEventA>().unwrap().len == basic_event_a().len);
        assert!(connector.get::<TestEventB>().is_none());
    }

    #[test]
    pub fn test_trigger() {
        let connector = connector();
        let mut count = 0;
        let v: Vec<&TypeId> = connector.trigger_events().inspect(|_| count += 1).collect();
        assert!(count == 1);
        assert!(v.get(0).unwrap() == &&TypeId::of::<TestEventA>())
    }
}
