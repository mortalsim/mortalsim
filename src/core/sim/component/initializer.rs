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

pub struct SimComponentInitializer<'a, 'b> {
    hub: &'a mut EventHub<'b>,
    connector: Rc<RefCell<SimConnector<'b>>>,
    component: Rc<RefCell<dyn SimComponent>>,
    pub(in super::super) listener_ids: Vec<IdType>,
    pub(in super::super) transformer_ids: Vec<IdType>,
    input_events: HashSet<TypeId>,
    output_events: HashSet<TypeId>,
}

impl<'a, 'b> SimComponentInitializer<'a, 'b> {
    pub fn new(hub: &'a mut EventHub<'b>, connector: Rc<RefCell<SimConnector<'b>>>, component: Rc<RefCell<dyn SimComponent>>) -> SimComponentInitializer<'a, 'b> {
        SimComponentInitializer {
            hub: hub,
            connector: connector,
            component: component,
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
        let component_weak = Rc::downgrade(&self.component);
        let connector_weak = Rc::downgrade(&self.connector);

        let listener_id = self.hub.on_prioritized(priority, move |evt: Arc<T>| {
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
    pub fn transform<T: Event>(&mut self, transformer: impl FnMut(&mut T) + 'b) {
        let transformer_id = self.hub.transform(transformer);
        self.transformer_ids.push(transformer_id);
    }
    
    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim` with a given priority value.
    /// 
    /// ### Arguments
    /// * `priority` - Transformation order priority for this registration
    /// * `transformer` - Function to modify the `Event`
    pub fn transform_prioritized<T: Event>(&mut self, priority: i32, transformer: impl FnMut(&mut T) + 'b) {
        let transformer_id = self.hub.transform_prioritized(priority, transformer);
        self.transformer_ids.push(transformer_id);
    }
    
    /// Sets an `Event` as the initial state on the `Sim`
    /// 
    /// ### Arguments
    /// * `event` - `Event` instance to set on initial state
    pub fn set_initial_state<T: Event>(&mut self, priority: i32, transformer: impl FnMut(&mut T) + 'b) {
        let transformer_id = self.hub.transform_prioritized(priority, transformer);
        self.transformer_ids.push(transformer_id);
    }
}