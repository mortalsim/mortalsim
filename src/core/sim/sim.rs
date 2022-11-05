use std::fmt;
use std::collections::{HashMap, HashSet, BTreeSet, VecDeque};
use std::any::TypeId;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use uuid::Uuid;
use anyhow::Result;
use either::Either;
use super::sim_state::SimState;
use super::component::{SimComponentInitializer, SimConnector, SimComponent};
use super::time_manager::{Time, TimeManager};
use crate::event::Event;
use crate::core::hub::event_transformer::{EventTransformer, TransformerItem};
use crate::core::hub::EventHub;
use crate::util::id_gen::{IdType, InvalidIdError};

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn SimComponent> + Send>>> = Mutex::new(HashMap::new());
}

/// Registers a Sim component. By default, the component will be added to all newly created Sim objects
///
/// ### Arguments
/// * `component_name` - String name for the component
/// * `factory`        - Factory function which creates an instance of the component
fn register_component(component_name: &'static str, factory: impl FnMut() -> Box<dyn SimComponent> + Send + 'static) {
    log::debug!("Registering component {}", component_name);
    COMPONENT_REGISTRY.lock().unwrap().insert(component_name, Box::new(factory));
}

pub trait Sim {
    /// Returns the current simulation time
    fn get_time(&self) -> Time;
    
    /// Determines if the given component name corresponds to an active component
    /// on this Sim
    fn has_component(&self, component_name: &'static str) -> bool;

    /// Retrieves the set of names of components which are active on this Sim
    fn active_components(&self) -> HashSet<&'static str>;

    /// Adds components to this Sim. Panics if any component names are invalid
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to add
    fn add_components(&mut self, component_names: HashSet<&'static str>);

    /// Removes a component from this Sim. Panics if any of the component names
    /// are invalid.
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to remove
    fn remove_components(&mut self, component_names: HashSet<&'static str>);

    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    /// 
    /// If there are no Events or listeners in the queue, time will remain unchanged
    fn advance(&mut self);

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    fn advance_by(&mut self, time_step: Time);

    /// Schedules an `Event` for future emission on this simulation
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    /// 
    /// Returns the schedule ID
    fn schedule_event(&mut self, wait_time: Time, event: impl Event) -> IdType;

    /// Unschedules a previously scheduled `Event`
    /// 
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    /// 
    /// Returns an Err Result if the provided ID is invalid
    fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<()>;
}

pub struct CoreSim {
    sim_id: Uuid,
    active_components: HashMap<&'static str, Box<dyn SimComponent>>,
    time_manager: TimeManager,
    state: SimState,
    event_transformers: HashMap<TypeId, Vec<Box<dyn EventTransformer>>>,
    component_notifications: HashMap<TypeId, Vec<(i32, &'static str)>>,
    extension_notifications: HashMap<Uuid, HashMap<TypeId, Vec<Arc<dyn Event>>>>,
    extension_type_map: HashMap<TypeId, HashSet<Uuid>>,
    connector_map: HashMap<&'static str, SimConnector>,
    transformer_id_map: HashMap<&'static str, Vec<IdType>>,
    transformer_type_map: HashMap<IdType, TypeId>,
    /// Map of pending updates for each component
    notify_map: HashMap<&'static str, HashSet<TypeId>>,
}

impl fmt::Debug for CoreSim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Simulation<{:?}> {{ time: {:?}, active_components: {:?}, state: {:?} }}",
            self.time_manager.get_time(),
            self.sim_id,
            self.active_components.keys(),
            self.state)
    }
}

impl CoreSim {
    /// Internal function for creating a Sim object with initial SimState
    fn get_object(initial_state: SimState) -> CoreSim {
        CoreSim {
            sim_id: Uuid::new_v4(),
            active_components: HashMap::new(),
            time_manager: TimeManager::new(),
            state: initial_state,
            event_transformers: HashMap::new(),
            component_notifications: HashMap::new(),
            extension_notifications: HashMap::new(),
            extension_type_map: HashMap::new(),
            connector_map: HashMap::new(),
            transformer_id_map: HashMap::new(),
            transformer_type_map: HashMap::new(),
            notify_map: HashMap::new(),
        }
    }

    /// Creates a Sim with the default set of components which is equal to all registered
    /// components at the time of execution.
    pub fn new() -> CoreSim {
        let mut sim = Self::get_object(SimState::new());
        let component_set = COMPONENT_REGISTRY.lock().unwrap().keys().cloned().collect();
        sim.setup(component_set);
        sim
    }
    
    /// Creates a Sim with custom components.
    ///
    /// ### Arguments
    /// * `component_set` - Set of components to add on initialization
    /// 
    /// Returns a new Sim object
    pub fn new_custom(component_set: HashSet<&'static str>) -> CoreSim {
        let mut sim = Self::get_object(SimState::new());
        sim.setup(component_set);
        sim
    }
    
    /// Creates a Sim with initial State
    ///
    /// ### Arguments
    /// * `initial_state` - Initial SimState for the Sim
    /// 
    /// Returns a new Sim object
    pub fn new_with_state(initial_state: SimState) -> CoreSim {
        let mut sim = Self::get_object(initial_state);
        let component_set = COMPONENT_REGISTRY.lock().unwrap().keys().cloned().collect();
        sim.setup(component_set);
        sim
    }
    
    /// Creates a custom Sim with initial State
    ///
    /// ### Arguments
    /// * `component_set` - Set of components to add on initialization
    /// * `initial_state` - Initial SimState for the Sim
    /// 
    /// Returns a new Sim object
    pub fn new_custom_with_state(component_set: HashSet<&'static str>, initial_state: SimState) -> CoreSim {
        let mut sim = Self::get_object(initial_state);
        sim.setup(component_set);
        sim
    }
    
    /// Attaches emitted events to this Sim's canonical state
    /// and initializes components
    fn setup(&mut self, component_set: HashSet<&'static str>) {
        self.init_components(component_set);
    }

    /// Internal function for initializing components on this Sim. If a component which has
    /// already been initialized is initialized again, it will be replaced by a new instance.
    /// Panics if any provided component name is invalid.
    ///
    /// ### Arguments
    /// * `component_names` - Set of component names to initialize
    pub(crate) fn init_components(&mut self, component_names: HashSet<&'static str>) {

        // Initialize each component
        for component_name in component_names.into_iter() {
            log::debug!("Initializing component \"{}\" on Sim", component_name);
            match COMPONENT_REGISTRY.lock().unwrap().get_mut(component_name) {
                None => panic!("Invalid component name provided: \"{}\"", component_name),
                Some(factory) => {
                    let mut component = factory();
                    let mut initializer = SimComponentInitializer::new();
                    component.init(&mut initializer);
                    
                    self.active_components.insert(component_name, component);

                    let connector = self.setup_component(component_name, initializer);
                    self.connector_map.insert(component_name, connector);
                }
            }
        }
    }

    /// handles internal registrations and initial outputs for components
    pub(crate) fn setup_component(&mut self, component_name: &'static str, initializer: SimComponentInitializer) -> SimConnector {
        let mut connector = SimConnector::new();
        let mut transformer_ids = Vec::new();
        for transformer in initializer.pending_transforms {
            transformer_ids.push(self.insert_transformer(transformer));
        }
        self.transformer_id_map.insert(component_name, transformer_ids);
        
        for (priority, evt) in initializer.pending_notifies {
            let type_id = evt.type_id();
            connector.local_state.put_state(type_id, evt.into());
            match self.component_notifications.get_mut(&type_id) {
                None => {
                    self.component_notifications.insert(type_id, vec![(priority, component_name)]);
                }
                Some(list) => {
                    list.push((priority, component_name));
                }
            }
        }

        // Clear taint
        connector.local_state.clear_taint();
        connector
    }

    fn insert_transformer(&mut self, transformer: Box<dyn EventTransformer>) -> IdType {
        let transformer_id = transformer.transformer_id();
        let type_key = transformer.type_id();

        match self.event_transformers.get(&type_key) {
            Some(transformers) => {
                match transformers.binary_search(&transformer) {
                    Ok(_) => panic!("Duplicate Transformer id {}", transformer.transformer_id()),
                    Err(pos) => {
                        self.event_transformers.get_mut(&type_key).unwrap().insert(pos, transformer);
                    }
                }
            },
            None => {
                self.event_transformers.insert(type_key, vec![transformer]);
            }
        }
        
        // Add the id -> type mapping for quick removal if needed later
        self.transformer_type_map.insert(transformer_id, type_key);

        transformer_id
    }

    /// Returns true if the given Event type exists on the Sim's current
    /// state, false otherwise
    pub fn has_state<E: Event>(&self) -> bool {
        self.state.has_state::<E>()
    }

    /// Retrieves a current `Event` object from state, if it exists
    pub fn get_state<E: Event>(&self) -> Option<Arc<E>> {
        match self.state.get_state_ref(&TypeId::of::<E>()) {
            None => None,
            Some(trait_evt) => {
                match trait_evt.downcast_arc::<E>() {
                    Ok(evt) => {
                        Some(evt)
                    }
                    Err(_) => {
                        panic!("Event unable to downcast properly! Something went horribly wrong...")
                    }
                }
            }
        }
    }

    /// Registers a transformer for a specific Event. 
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    /// 
    /// Returns the registration ID for the transformer
    pub fn transform<E: Event>(&mut self, handler: impl FnMut(&mut E) + 'static) -> IdType {
        self.insert_transformer(Box::new(TransformerItem::new(handler)))
    }
    
    /// Registers a transformer for a specific Event with the given priority. Higher
    /// priority transformers are executed first.
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    /// * `priority` - Priority of the transformer
    /// 
    /// Returns the registration ID for the transformer
    pub fn transform_prioritized<E: Event>(&mut self, priority: i32, handler: impl FnMut(&mut E) + 'static) -> IdType {
        self.insert_transformer(Box::new(TransformerItem::new_prioritized(handler, priority)))
    }

    /// Unregisters a transformer for a specific Event with the given registration ID returned
    /// from the call to `transform` or `transform_prioritized`.
    ///
    /// ### Arguments
    /// * `transformer_id` - transformer registration ID
    /// 
    /// Returns Ok if successful, or Err if the provided ID is invalid.
    pub fn unset_transform(&mut self, transformer_id: IdType) -> Result<()> {
        match self.transformer_type_map.get(&transformer_id) {
            Some(type_key) => {
                let transformers = self.event_transformers.get_mut(type_key).unwrap();
                match transformers.iter().position(|l| l.transformer_id() == transformer_id) {
                    Some(pos) => {
                        transformers.remove(pos);
                        self.transformer_type_map.remove(&transformer_id);
                        Ok(())
                    },
                    None => Err(anyhow::Error::new(InvalidIdError::new(format!("{:?}", self), transformer_id)))
                }
            },
            None => Err(anyhow::Error::new(InvalidIdError::new(format!("{:?}", self), transformer_id)))
        }
    }

    fn execute_time_step(&mut self) {
        // Keep going until no more events / listeners are left to deal with
        loop {
            let next_events = self.time_manager.next_events();

            // If we have both, we need to determine which ones to deal with first
            // based on their scheduled time
            if next_events.is_some() {
                let (_, evt_list) = next_events.unwrap();
                for mut evt in evt_list {
                    // Call any transformers on the event
                    for transformers in self.event_transformers.get_mut(&evt.type_id()).iter_mut() {
                        for transformer in transformers.iter_mut() {
                            transformer.transform(evt.as_mut());
                        }
                    }
                    // Set it on the sim's state
                    self.state.put_state(evt.type_id(), evt.into());

                    // Stage it up for any listening extensions
                }
            }
            else {
                // break out of the loop when there's nothing left to process
                break;
            }
        }

        let mut notify_map = HashMap::new();
        
        // Now set the notify map for each component notification
        for type_id in self.state.get_tainted().clone() {
            match self.component_notifications.get(&type_id) {
                None => {},
                Some(notify_list) => {
                    for (_, component_name) in notify_list {
                        notify_map.entry(*component_name).or_insert(HashSet::new()).insert(type_id);
                    }
                }
            };
        }

        // Make sure we clear state taint so these updates are effectively
        // marked as "completed"
        self.state.clear_taint();

        // Update locally managed components
        self.update_components(notify_map);
    }

    fn update_components(&mut self, notify_map: HashMap<&'static str, HashSet<TypeId>>) {
        for (component_name, notify_set) in notify_map {
            // Need to remove the connector to avoid multiple mutable borrows of self
            match self.connector_map.remove(component_name) {
                Some(mut connector) => {
                    self.prepare_connector(component_name, &mut connector);
                    
                    // Execute component logic
                    self.active_components.get_mut(component_name).unwrap().run(&mut connector);

                    // Process the results
                    self.process_connector(&mut connector);

                    // Insert the connector back into our map
                    self.connector_map.insert(component_name, connector);
                }
                None => {
                    // This component is not managed internally, so we need to
                    // add it to the notify map field for an extension to update
                    match self.notify_map.get_mut(component_name) {
                        Some(set) => {
                            set.extend(notify_set)
                        }
                        None => {
                            self.notify_map.insert(component_name, notify_set);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn pending_updates<'a>(&'a mut self) -> impl Iterator<Item = &'static str> + 'a {
        self.notify_map.keys().map(|n| { *n })
    }

    pub(crate) fn clear_notifications(&mut self) {
        self.notify_map.clear()
    }

    pub(crate) fn prepare_connector(&mut self, component_name: &'static str, connector: &mut SimConnector) {
        // Update connector before component execution
        connector.trigger_events = {
            let notify_ids = self.notify_map.remove(component_name).unwrap_or(HashSet::new());
            notify_ids.iter().map(|id| { self.state.get_state_ref(id).unwrap() }).collect()
        };
        connector.sim_time = self.time_manager.get_time();
        connector.local_state.merge_tainted(&self.state);
    }

    pub(crate) fn process_connector(&mut self, connector: &mut SimConnector) {

        // Unschedule any requested events
        if connector.unschedule_all {
            for (_, id_map) in connector.scheduled_events.drain() {
                for (schedule_id, _) in id_map {
                    self.time_manager.unschedule_event(&schedule_id).unwrap();
                }
            }
        }
        else {
            for schedule_id in connector.pending_unschedules.drain(..) {
                self.time_manager.unschedule_event(&schedule_id).unwrap();
                let type_id = connector.schedule_id_type_map.remove(&schedule_id).unwrap();
                connector.scheduled_events.remove(&type_id).unwrap();
            }
        }

        // Schedule any new events
        for (wait_time, evt) in connector.pending_schedules.drain(..) {
            let type_id = evt.type_id();
            let schedule_id = self.time_manager.schedule_event(wait_time, evt);
            connector.schedule_id_type_map.insert(schedule_id, type_id);
            match connector.scheduled_events.get_mut(&type_id) {
                None => {
                    let mut map = HashMap::new();
                    map.insert(schedule_id, wait_time);
                    connector.scheduled_events.insert(type_id, map);
                },
                Some(map) => {
                    map.insert(schedule_id, wait_time);
                }
            }
        }
    }

    pub(crate) fn notify_extension<E: Event>(&mut self, extension_id: Uuid) {
        let ext_notify_map = self.extension_notifications.entry(extension_id).or_insert(HashMap::new());
        ext_notify_map.insert(TypeId::of::<E>(), Vec::new());
        self.extension_type_map.entry(TypeId::of::<E>()).or_insert(HashSet::new()).insert(extension_id);
    }
    
    pub(crate) fn extension_events<'a, E: Event>(&'a mut self, extension_id: &Uuid) -> impl Iterator<Item = Arc<E>> + 'a {
        match self.extension_notifications.get_mut(extension_id) {
            Some(notifications) => {
                match notifications.get_mut(&TypeId::of::<E>()) {
                    Some(evt_list) => {
                        Either::Left(evt_list.drain(..).map(|e| { e.downcast_arc::<E>().unwrap() }))
                    },
                    None => Either::Right(std::iter::empty())
                }
            },
            None => Either::Right(std::iter::empty())
        }
    }
}

impl Sim for CoreSim {

    /// Returns the current simulation time
    fn get_time(&self) -> Time {
        self.time_manager.get_time()
    }

    /// Determines if the given component name corresponds to an active component
    /// on this Sim
    fn has_component(&self, component_name: &'static str) -> bool {
        return self.active_components.contains_key(component_name)
    }

    /// Retrieves the set of names of components which are active on this Sim
    fn active_components(&self) -> HashSet<&'static str> {
        self.active_components.keys().cloned().collect()
    }

    /// Adds components to this Sim. Panics if any component names are invalid
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to add
    fn add_components(&mut self, component_names: HashSet<&'static str>) {
        self.init_components(component_names);
    }

    /// Removes a component from this Sim. Panics if any of the component names
    /// are invalid.
    ///
    /// ### Arguments
    /// * `component_names` - Set of components to remove
    fn remove_components(&mut self, component_names: HashSet<&'static str>) {
        for component_name in component_names {
            if self.active_components.remove(component_name).is_none() {
                panic!("Invalid component name \"{}\" provided for removal", component_name);
            }
        }
    }


    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    /// 
    /// If there are no Events or listeners in the queue, time will remain unchanged
    fn advance(&mut self) {
        self.time_manager.advance();
        self.execute_time_step();
    }

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    fn advance_by(&mut self, time_step: Time) {
        self.time_manager.advance_by(time_step);
        self.execute_time_step();
    }


    /// Schedules an `Event` for future emission on this simulation
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    /// 
    /// Returns the schedule ID
    fn schedule_event(&mut self, wait_time: Time, event: impl Event) -> IdType {
        self.time_manager.schedule_event(wait_time, Box::new(event))
    }

    /// Unschedules a previously scheduled `Event`
    /// 
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    /// 
    /// Returns an Err Result if the provided ID is invalid
    fn unschedule_event(&mut self, schedule_id: &IdType) -> Result<()> {
        self.time_manager.unschedule_event(schedule_id)
    }
}
