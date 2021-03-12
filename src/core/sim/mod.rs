mod time_manager;
mod sim_state;
pub mod component;

use std::collections::HashMap;
use std::any::TypeId;
use std::sync::Mutex;
use std::rc::Rc;
use std::cell::RefCell;
use time_manager::TimeManager;
use sim_state::SimState;
pub use component::{BioConnector, BioComponent};
use crate::core::hub::EventHub;

lazy_static! {
    static ref COMPONENT_MAP: Mutex<HashMap<&'static str, Box<dyn FnMut() -> Box<dyn BioComponent> + Send>>> = Mutex::new(HashMap::new());
}

pub struct Sim<'a> {
    components: Vec<(BioConnector<'a>, Box<dyn BioComponent>)>,
    hub: Rc<RefCell<EventHub<'a>>>,
    time_manager: Rc<RefCell<TimeManager<'a>>>,
}

impl<'a> Sim<'a> {
    fn register(component_name: &'static str, factory: impl FnMut() -> Box<dyn BioComponent> + Send + 'static) {
        log::debug!("Registering component {}", component_name);
        COMPONENT_MAP.lock().unwrap().insert(component_name, Box::new(factory));
    }

    fn new() -> Sim<'a> {
        let mut components = Vec::new();
        let hub = Rc::new(RefCell::new(EventHub::new()));
        let time_manager = Rc::new(RefCell::new(TimeManager::new()));

        for (component_name, factory) in COMPONENT_MAP.lock().unwrap().iter_mut() {
            log::debug!("Adding component \"{}\" to new Sim", component_name);
            components.push((BioConnector::new(time_manager.clone(), hub.clone()), factory()))
        }

        Sim {
            components: components,
            hub: hub,
            time_manager: time_manager,
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
        Sim::register("TestComponent", TestComponent::factory);
        let sim = Sim::new();
    }

}