mod time_manager;
mod sim_state;
pub mod component;

use std::collections::{HashMap, HashSet, VecDeque};
use std::any::TypeId;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use uuid::Uuid;
use anyhow::Result;
use time_manager::TimeManager;
use sim_state::SimState;
use crate::event::Event;
pub use component::{SimComponentInitializer, SimConnector, SimComponent};
pub use time_manager::Time;
use crate::core::hub::EventHub;
use crate::util::id_gen::IdType;

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn SimComponent> + Send>>> = Mutex::new(HashMap::new());
}

pub struct Sim<'a> {
    sim_id: Uuid,
    active_components: HashMap<&'static str, SimComponentInitializer<'a>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    state: Rc<RefCell<SimState>>,
}

impl<'a> Sim<'a> {
    /// Registers a Sim component. By default, the component will be added to all newly created Sim objects
    ///
    /// ### Arguments
    /// * `component_name` - String name for the component
    /// * `factory`        - Factory function which creates an instance of the component
    pub fn register_component(component_name: &'static str, factory: impl FnMut() -> Box<dyn SimComponent> + Send + 'static) {
        log::debug!("Registering component {}", component_name);
        COMPONENT_REGISTRY.lock().unwrap().insert(component_name, Box::new(factory));
    }
    
    /// Internal function for creating a Sim object with initial SimState
    fn get_object(initial_state: SimState) -> Sim<'a> {
        Sim {
            sim_id: Uuid::new_v4(),
            active_components: HashMap::new(),
            hub: Rc::new(RefCell::new(EventHub::new())),
            time_manager: Rc::new(RefCell::new(TimeManager::new())),
            state: Rc::new(RefCell::new(initial_state)),
        }
    }

    /// Creates a Sim with the default set of components which is equal to all registered
    /// components at the time of execution.
    pub fn new() -> Sim<'a> {
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
    pub fn new_custom(component_set: HashSet<&'static str>) -> Sim<'a> {
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
    pub fn new_with_state(initial_state: SimState) -> Sim<'a> {
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
    pub fn new_custom_with_state(component_set: HashSet<&'static str>, initial_state: SimState) -> Sim<'a> {
        let mut sim = Self::get_object(initial_state);
        sim.setup(component_set);
        sim
    }

    /// Initial setup for the simulation
    fn setup(&mut self, component_names: HashSet<&'static str>) {
        self.init_components(component_names);
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

    /// Internal function for initializing components on this Sim. If a component which has
    /// already been initialized is initialized again, it will be replaced by a new instance.
    /// Panics if any provided component name is invalid.
    ///
    /// ### Arguments
    /// * `component_names` - Set of component names to initialize
    fn init_components(&mut self, component_names: HashSet<&'static str>) {

        // Initialize each component
        for component_name in component_names.into_iter() {
            log::debug!("Initializing component \"{}\" on Sim", component_name);
            match COMPONENT_REGISTRY.lock().unwrap().get_mut(component_name) {
                None => panic!("Invalid component name provided: \"{}\"", component_name),
                Some(factory) => {
                    let component = factory();
                    let component_ref = Rc::new(RefCell::new(component));
                    let mut initializer = SimComponentInitializer::new(self.time_manager.clone(), self.hub.clone(), component_ref.clone());
                    component_ref.borrow_mut().init(&mut initializer);

                    // set initial state
                    self.state.borrow_mut().merge_tainted(&initializer.connector.borrow().local_state);
                    initializer.connector.borrow_mut().local_state.clear_taint();

                    self.active_components.insert(component_name, initializer);
                }
            }
        }

        // Set state for each component
        for (_, initializer) in self.active_components.iter_mut() {

            // Merge the canonical Sim state to the component's local state
            initializer.connector.borrow_mut().local_state.merge_all(&self.state.borrow());
        }
    }

    /// Returns true if the given Event type exists on the Sim's current
    /// state, false otherwise
    pub fn has_state<T: Event>(&self) -> bool {
        self.state.borrow().has_state::<T>()
    }

    /// Retrieves a current `Event` object from state, if it exists
    pub fn get_state<T: Event>(&self) -> Option<Arc<T>> {
        match self.state.borrow().get_state_ref(&TypeId::of::<T>()) {
            None => None,
            Some(trait_evt) => {
                match trait_evt.downcast_arc::<T>() {
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

    /// Dispatches an Event. First calls any registered transformers for the
    /// Event, then passes the event to all listeners.
    ///
    /// ### Arguments
    /// * `evt` - Event to dispatch
    pub fn emit<T: Event>(&mut self, evt: T) {
        self.hub.borrow_mut().emit(evt)
    }
    
    /// Registers a listener for any Event. 
    ///
    /// ### Arguments
    /// * `handler` - Event handling function
    /// 
    /// Returns the registration ID for the listener
    pub fn on_any(&mut self, listener: impl FnMut(Arc<dyn Event>) + 'a) -> IdType {
        self.hub.borrow_mut().on_any(listener)
    }

    /// Registers a listener for any Event with the given priority value. Higher
    /// priority listeners are executed first.
    ///
    /// ### Arguments
    /// * `priority` - Priority of the listener
    /// * `handler` - Event handling function
    /// 
    /// Returns the registration ID for the listener
    pub fn on_any_prioritized(&mut self, priority: i32, handler: impl FnMut(Arc<dyn Event>) + 'a) -> IdType {
        self.hub.borrow_mut().on_any_prioritized(priority, handler)
    }
    
    /// Unregisters a listener for any Event with the given registration ID returned
    /// from the call to `on_any` or `on_any_prioritized`.
    ///
    /// ### Arguments
    /// * `listener_id` - listener registration ID
    /// 
    /// Returns Ok if successful, or Err if the provided ID is invalid.
    pub fn off_any(&mut self, listener_id: IdType) -> Result<()> {
        self.hub.borrow_mut().off_any(listener_id)
    }

    /// Registers a listener for a specific Event. 
    ///
    /// ### Arguments
    /// * `handler` - Event handling function
    /// 
    /// Returns the registration ID for the listener
    pub fn on<T: Event>(&mut self, handler: impl FnMut(Arc<T>) + 'a) -> IdType {
        self.hub.borrow_mut().on(handler)
    }
    
    /// Registers a listener for a specific Event with the given priority value.
    /// Higher priority listeners are executed first.
    ///
    /// ### Arguments
    /// * `priority` - Priority of the listener
    /// * `handler` - Event handling function
    /// 
    /// Returns the registration ID for the listener
    pub fn on_prioritized<T: Event>(&mut self, priority: i32, handler: impl FnMut(Arc<T>) + 'a) -> IdType {
        self.hub.borrow_mut().on_prioritized(priority, handler)
    }

    /// Unregisters a listener for a specific Event with the given registration ID returned
    /// from the call to `on` or `on_prioritized`.
    ///
    /// ### Arguments
    /// * `listener_id` - listener registration ID
    /// 
    /// Returns Ok if successful, or Err if the provided ID is invalid.
    pub fn off(&mut self, listener_id: IdType) -> Result<()> {
        self.hub.borrow_mut().off(listener_id)
    }
    
    /// Registers a transformer for a specific Event. 
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    /// 
    /// Returns the registration ID for the transformer
    pub fn transform<T: Event>(&mut self, handler: impl FnMut(&mut T) + 'a) -> IdType {
        self.hub.borrow_mut().transform(handler)
    }
    
    /// Registers a transformer for a specific Event with the given priority. Higher
    /// priority transformers are executed first.
    ///
    /// ### Arguments
    /// * `handler` - Event transforming function
    /// * `priority` - Priority of the transformer
    /// 
    /// Returns the registration ID for the transformer
    pub fn transform_prioritized<T: Event>(&mut self, priority: i32, handler: impl FnMut(&mut T) + 'a) -> IdType {
        self.hub.borrow_mut().transform_prioritized(priority, handler)
    }

    /// Unregisters a transformer for a specific Event with the given registration ID returned
    /// from the call to `transform` or `transform_prioritized`.
    ///
    /// ### Arguments
    /// * `transformer_id` - transformer registration ID
    /// 
    /// Returns Ok if successful, or Err if the provided ID is invalid.
    pub fn unset_transform(&mut self, transformer_id: IdType) -> Result<()> {
        self.hub.borrow_mut().unset_transform(transformer_id)
    }

    /// Returns the current simulation time
    pub fn get_time(&self) -> Time {
        self.time_manager.borrow().get_time()
    }

    /// Advances simulation time to the next `Event` or listener in the queue, if any.
    /// 
    /// If there are no Events or listeners in the queue, time will remain unchanged
    pub fn advance(&mut self) {
        self.time_manager.borrow_mut().advance();
    }

    /// Advances simulation time by the provided time step
    /// 
    /// If a negative value is provided, time will immediately jump to
    /// the next scheduled Event, if any.
    /// 
    /// ### Arguments
    /// * `time_step` - Amount of time to advance by
    pub fn advance_by(&mut self, time_step: Time) {
        self.time_manager.borrow_mut().advance_by(time_step);

    }

    fn execute_time_step(&mut self) {
        // Keep going until no more events / listeners are left to deal with
        loop {
            let next_events = self.time_manager.borrow_mut().next_events();
            let next_listeners = self.time_manager.borrow_mut().next_listeners();

            // If we have both, we need to determine which ones to deal with first
            // based on their scheduled time
            if next_events.is_some() && next_listeners.is_some() {
                let (evt_time, evt_list) = next_events.unwrap();
                let (lis_time, lis_list) = next_listeners.unwrap();

                if evt_time <= lis_time {
                    for (type_key, evt) in evt_list.into_iter() {
                        self.hub.borrow_mut().emit_typed(type_key, evt);
                    }
                    for listener in lis_list {
                        listener();
                    }
                }
                else {
                    for listener in lis_list {
                        listener();
                    }
                    for (type_key, evt) in evt_list.into_iter() {
                        self.hub.borrow_mut().emit_typed(type_key, evt);
                    }
                }
            }
            else if next_events.is_some() {
                let (_, evt_list) = next_events.unwrap();
                for (type_key, evt) in evt_list.into_iter() {
                    self.hub.borrow_mut().emit_typed(type_key, evt);
                }
            }
            else if next_listeners.is_some() {
                let (_, lis_list) = next_listeners.unwrap();
                for listener in lis_list {
                    listener();
                }
            }
            else {
                // break out of the loop when there's nothing left to process
                break;
            }
        }
    }

    /// Schedules an `Event` for future emission on this simulation
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before emitting the Event
    /// * `event` - Event instance to emit
    /// 
    /// Returns the schedule ID
    pub fn schedule_event<T: Event>(&mut self, wait_time: Time, event: T) -> IdType {
        self.time_manager.borrow_mut().schedule_event(wait_time, event)
    }

    /// Unschedules a previously scheduled `Event`
    /// 
    /// ### Arguments
    /// * `schedule_id` - Schedule ID returned by `schedule_event`
    /// 
    /// Returns an Err Result if the provided ID is invalid
    pub fn unschedule_event(&mut self, schedule_id: IdType) -> Result<()> {
        self.time_manager.borrow_mut().unschedule_event(schedule_id)
    }

    /// Registers a listener for time advances
    /// 
    /// ### Arguments
    /// * `listener` - function to call when time advances
    pub fn on_advance(&mut self, listener: impl FnMut() + 'a) -> IdType {
        self.time_manager.borrow_mut().on_advance(listener)
    }

    /// Unregisters a previously attached time advance listener
    /// 
    /// ### Arguments
    /// * `listener_id` - identifier returned from the call to `on_advance`
    /// 
    /// Returns an `Err` if the provided listener_id is invalid
    pub fn off_advance(&mut self, listener_id: IdType) -> Result<()> {
        self.time_manager.borrow_mut().off_advance(listener_id)
    }

    /// Schedules a callback to be called at a future simulation time
    /// 
    /// ### Arguments
    /// * `wait_time` - amount of simulation time to wait before calling the listener
    /// * `listener` - function to call at the scheduled time
    /// 
    /// Returns an ID for the scheduled listener
    pub fn schedule_callback(&mut self, wait_time: Time, listener: impl FnOnce() + 'a) -> IdType {
        self.time_manager.borrow_mut().schedule_callback(wait_time, listener)
    }

    /// Unschedules a previously scheduled listener
    /// 
    /// ### Arguments
    /// * `listener_id` - The identifier returned from the call to `schedule_callback`
    /// 
    /// Returns an `Err` if the provided listener_id is invalid
    pub fn unschedule_callback(&mut self, listener_id: IdType) -> Result<()> {
        self.time_manager.borrow_mut().unschedule_callback(listener_id)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::Sim;
    use super::component::SimComponent;
    use super::component::test::TestComponentA;

    #[test]
    fn test_registry() {
        crate::test::init_test();
        Sim::register_component("TestComponent", TestComponentA::factory);
        Sim::new();
    }

}