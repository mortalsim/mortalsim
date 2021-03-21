use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::cell::RefCell;
use std::any::TypeId;
use std::collections::HashSet;
use crate::core::hub::EventHub;
use crate::core::sim::{TimeManager, SimState};
use crate::event::Event;
use crate::util::id_gen::IdType;
use super::{SimComponent, SimConnector};

pub struct SimComponentInitializer<'a> {
    pub(in super::super) connector: Rc<RefCell<SimConnector<'a>>>,
    component: Rc<RefCell<Box<dyn SimComponent>>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    listener_ids: Vec<IdType>,
    transformer_ids: Vec<IdType>,
    input_events: HashSet<TypeId>,
    output_events: HashSet<TypeId>,
}

impl<'a> SimComponentInitializer<'a> {
    pub fn new(time_manager: Rc<RefCell<TimeManager<'a>>>, hub: Rc<RefCell<EventHub<'a>>>, component: Rc<RefCell<Box<dyn SimComponent>>>) -> SimComponentInitializer<'a> {
        SimComponentInitializer {
            connector: Rc::new(RefCell::new(SimConnector::new(time_manager.clone()))),
            component: component,
            hub: hub,
            listener_ids: Vec::new(),
            transformer_ids: Vec::new(),
            input_events: HashSet::new(),
            output_events: HashSet::new(),
        }
    }

    /// Registers the corresponding `SimComponent` to `run` whenever the
    /// provided `Event` is modified on the `Sim`.
    /// 
    /// ### Arguments
    /// * `default` - Default `Event` value when one isn't provided by another component
    pub fn notify<T: Event>(&mut self, default: T) {
        self.notify_prioritized::<T>(0, default);
    }
    
    /// Registers the corresponding `SimComponent` to `run` whenever the
    /// provided `Event` is modified on the `Sim` with a given priority value.
    /// 
    /// ### Arguments
    /// * `priority` - Notify order priority for this registration
    /// * `default` - Default `Event` value when one isn't provided by another component
    pub fn notify_prioritized<T: Event>(&mut self, priority: i32, default: T) {
        let type_key = TypeId::of::<T>();
        // If this event type has already been registered as an output, panic
        if self.output_events.contains(&type_key) {
            panic!("Components cannot run against Events they are producing! This could cause an infinite loop.")
        }

        // Set the provided default
        self.connector.borrow_mut().local_state.set_state_quiet(default);

        // Create weak pointers to our connector & component
        let connector_weak = Rc::downgrade(&self.connector);
        let component_weak = Rc::downgrade(&self.component);

        let listener_id = self.hub.borrow_mut().on_prioritized(priority, move |evt: Arc<T>| {
            match component_weak.upgrade() {
                Some(component) => {
                    match connector_weak.upgrade() {
                        Some(connector) => {
                            let mut conn = connector.borrow_mut();
                            conn.local_state.put_state(type_key, evt.clone());
                            conn.set_trigger(evt);
                            component.borrow_mut().run(&mut conn);
                        },
                        None => {}
                    }
                },
                None => {}
            }
        });
        self.listener_ids.push(listener_id);
    }
    
    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim`.
    /// 
    /// ### Arguments
    /// * `transformer` - Function to modify the `Event`
    pub fn transform<T: Event>(&mut self, transformer: impl FnMut(&mut T) + 'a) {
        let transformer_id = self.hub.borrow_mut().transform(transformer);
        self.transformer_ids.push(transformer_id);
    }
    
    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim` with a given priority value.
    /// 
    /// ### Arguments
    /// * `priority` - Transformation order priority for this registration
    /// * `transformer` - Function to modify the `Event`
    pub fn transform_prioritized<T: Event>(&mut self, priority: i32, transformer: impl FnMut(&mut T) + 'a) {
        let transformer_id = self.hub.borrow_mut().transform_prioritized(priority, transformer);
        self.transformer_ids.push(transformer_id);
    }
    
    /// Sets an `Event` as the initial state on the `Sim`
    /// 
    /// ### Arguments
    /// * `event` - `Event` instance to set on initial state
    pub fn set_initial_state<T: Event>(&mut self, priority: i32, transformer: impl FnMut(&mut T) + 'a) {
        let transformer_id = self.hub.borrow_mut().transform_prioritized(priority, transformer);
        self.transformer_ids.push(transformer_id);
    }
}

// Unset any listeners & transformers when this object drops
impl<'a> Drop for SimComponentInitializer<'a> {
    fn drop(&mut self) {
        let mut hub = self.hub.borrow_mut();
        for listener_id in self.listener_ids.iter_mut() {
            match hub.off(*listener_id) {
                Err(err) => panic!(err),
                Ok(_) => {}
            }
        }

        for transformer_id in self.transformer_ids.iter_mut() {
            match hub.unset_transform(*transformer_id) {
                Err(err) => panic!(err),
                Ok(_) => {}
            }
        }
    }
}