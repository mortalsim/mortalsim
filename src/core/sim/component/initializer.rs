use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::cell::RefCell;
use std::any::TypeId;
use crate::core::hub::EventHub;
use crate::core::sim::{TimeManager, SimState};
use crate::event::Event;
use crate::util::id_gen::IdType;
use super::{BioComponent, BioConnector};

pub struct BioComponentInitializer<'a> {
    pub(in super::super) connector: Rc<RefCell<BioConnector<'a>>>,
    component: Rc<RefCell<Box<dyn BioComponent>>>,
    hub: Rc<RefCell<EventHub<'a>>>,
    listener_ids: Vec<IdType>,
    transformer_ids: Vec<IdType>,
}

impl<'a> BioComponentInitializer<'a> {
    pub fn new(time_manager: Rc<RefCell<TimeManager<'a>>>, hub: Rc<RefCell<EventHub<'a>>>, component: Rc<RefCell<Box<dyn BioComponent>>>) -> BioComponentInitializer<'a> {
        BioComponentInitializer {
            connector: Rc::new(RefCell::new(BioConnector::new(time_manager.clone()))),
            component: component,
            hub: hub,
            listener_ids: Vec::new(),
            transformer_ids: Vec::new(),
        }
    }

    pub fn notify<T: Event>(&mut self, default: T) {
        self.notify_prioritized::<T>(0, default);
    }
    
    pub fn notify_prioritized<T: Event>(&mut self, priority: i32, default: T) {
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
                            conn.local_state.put_state(TypeId::of::<T>(), evt.clone());
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
    
    pub fn transform<T: Event>(&mut self, transformer: impl FnMut(&mut T) + 'a) {
        let transformer_id = self.hub.borrow_mut().transform(transformer);
        self.transformer_ids.push(transformer_id);
    }
    
    pub fn transform_prioritized<T: Event>(&mut self, priority: i32, transformer: impl FnMut(&mut T) + 'a) {
        let transformer_id = self.hub.borrow_mut().transform_prioritized(priority, transformer);
        self.transformer_ids.push(transformer_id);
    }
}

// Unset any listeners & transformers when this object drops
impl<'a> Drop for BioComponentInitializer<'a> {
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