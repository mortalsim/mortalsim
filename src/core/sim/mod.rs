mod time_manager;
mod sim_state;
pub mod component;

use std::collections::{HashMap, HashSet, VecDeque};
use std::any::TypeId;
use std::sync::{Mutex, Arc};
use std::rc::Rc;
use std::cell::RefCell;
use time_manager::TimeManager;
use sim_state::SimState;
pub use component::{BioComponentInitializer, BioConnector, BioComponent};
pub use time_manager::Time;
use crate::core::hub::EventHub;

lazy_static! {
    static ref COMPONENT_REGISTRY: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn BioComponent> + Send>>> = Mutex::new(HashMap::new());
}

pub struct Sim<'a> {
    initializers: Vec<BioComponentInitializer<'a>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    time_manager: Rc<RefCell<TimeManager<'a>>>,
    state: SimState,
}

impl<'a> Sim<'a> {
    fn get_object() -> Sim<'a> {
        Sim {
            initializers: Vec::new(),
            hub: Rc::new(RefCell::new(EventHub::new())),
            time_manager: Rc::new(RefCell::new(TimeManager::new())),
            state: SimState::new(),
        }
    }

    pub fn register_component(component_name: &'static str, factory: impl FnMut() -> Box<dyn BioComponent> + Send + 'static) {
        log::debug!("Registering component {}", component_name);
        COMPONENT_REGISTRY.lock().unwrap().insert(component_name, Box::new(factory));
    }

    pub fn new() -> Sim<'a> {
        let mut sim = Self::get_object();

        // build our list of components
        let mut component_list = Vec::new();
        for (component_name, factory) in COMPONENT_REGISTRY.lock().unwrap().iter_mut() {
            log::debug!("Adding component \"{}\" to new Sim", component_name);
            component_list.push(factory());
        }

        sim.init(component_list);

        sim
    }
    
    pub fn new_custom(component_set: HashSet<&str>) -> Sim<'a> {
        let mut sim = Self::get_object();
        
        // build our list of components
        let mut component_list = Vec::new();
        for component_name in component_set {
            match COMPONENT_REGISTRY.lock().unwrap().get_mut(component_name) {
                Some(factory) => {
                    log::debug!("Adding component \"{}\" to new Sim", component_name);
                    component_list.push(factory());
                }
                None => {
                    panic!("Invalid component name provided: \"{}\"", component_name);
                }
            }
        }

        sim.init(component_list);

        sim
    }

    fn init(&mut self, component_list: Vec<Box<dyn BioComponent>>) {

        // Initialize each component
        for component in component_list.into_iter() {
            let component_ref = Rc::new(RefCell::new(component));
            let mut initializer = BioComponentInitializer::new(self.time_manager.clone(), self.hub.clone(), component_ref.clone());
            component_ref.borrow_mut().init(&mut initializer);

            // set initial state
            self.state.merge_tainted(&initializer.connector.borrow().local_state);
            initializer.connector.borrow_mut().local_state.clear_taint();

            self.initializers.push(initializer);
        }

        // Create the runtime connector for each component and set initial state
        for initializer in self.initializers.iter_mut() {

            // Merge the canonical Sim state to the component's local state
            initializer.connector.borrow_mut().local_state.merge_all(&self.state);
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