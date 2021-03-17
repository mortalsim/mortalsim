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
pub use component::{BioComponentInitializer, BioConnector, BioComponent};
pub use time_manager::Time;
use crate::core::hub::EventHub;

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn BioComponent> + Send>>> = Mutex::new(HashMap::new());
}

pub struct Sim<'a> {
    sim_id: Uuid,
    active_components: HashMap<&'static str, BioComponentInitializer<'a>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    state: Rc<RefCell<SimState>>,
}

impl<'a> Sim<'a> {
    fn get_object() -> Sim<'a> {
        Sim {
            sim_id: Uuid::new_v4(),
            active_components: HashMap::new(),
            hub: Rc::new(RefCell::new(EventHub::new())),
            time_manager: Rc::new(RefCell::new(TimeManager::new())),
            state: Rc::new(RefCell::new(SimState::new())),
        }
    }

    pub fn register_component(component_name: &'static str, factory: impl FnMut() -> Box<dyn BioComponent> + Send + 'static) {
        log::debug!("Registering component {}", component_name);
        COMPONENT_REGISTRY.lock().unwrap().insert(component_name, Box::new(factory));
    }

    pub fn new() -> Sim<'a> {
        let mut sim = Self::get_object();

        // our list of components is all that currently exist in the registry by default
        let components = COMPONENT_REGISTRY.lock().unwrap().keys().cloned().collect();
        sim.init(components);

        sim
    }
    
    pub fn new_custom(component_set: HashSet<&'static str>) -> Sim<'a> {
        let mut sim = Self::get_object();
        sim.init(component_set);
        sim
    }

    fn active_components(&self) -> HashSet<&'static str> {
        self.active_components.keys().cloned().collect()
    }

    fn add_component(&mut self, component_name: &'static str) {
        let mut set = HashSet::new();
        set.insert(component_name);
        self.init(set);
    }
    
    fn add_components(&mut self, component_names: HashSet<&'static str>) {
        self.init(component_names);
    }

    fn remove_component(&mut self, component_name: &'static str) -> Result<()> {
        match self.active_components.remove(component_name) {
            Some(_) => Ok(()),
            None => Err(anyhow!("Invalid component name \"{}\" provided for removal", component_name))
        }
    }

    fn init(&mut self, component_names: HashSet<&'static str>) {

        // Initialize each component
        for component_name in component_names.into_iter() {
            log::debug!("Initializing component \"{}\" on Sim", component_name);
            match COMPONENT_REGISTRY.lock().unwrap().get_mut(component_name) {
                None => panic!("Invalid component name provided: \"{}\"", component_name),
                Some(factory) => {
                    let component = factory();
                    let component_ref = Rc::new(RefCell::new(component));
                    let mut initializer = BioComponentInitializer::new(self.time_manager.clone(), self.hub.clone(), component_ref.clone());
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

    /// Retrieves the current `Event` object from state
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
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::Sim;
    use super::component::BioComponent;
    use super::component::test::TestComponent;

    #[test]
    fn test_registry() {
        crate::test::init_test();
        Sim::register_component("TestComponent", TestComponent::factory);
        Sim::new();
    }

}