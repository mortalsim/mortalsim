use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::cell::RefCell;
use std::any::TypeId;
use std::collections::HashSet;
use crate::core::sim::{TimeManager, SimState};
use crate::core::hub::EventHub;
use crate::core::hub::event_transformer::{EventTransformer, TransformerItem};
use crate::event::Event;
use crate::util::id_gen::IdType;
use super::{SimComponent, SimConnector};

pub struct SimComponentInitializer {
    pub(crate) input_events: HashSet<TypeId>,
    pub(crate) output_events: HashSet<TypeId>,
    pub(crate) pending_notifies: Vec<(i32, Box<dyn Event>)>,
    pub(crate) pending_transforms: Vec<Box<dyn EventTransformer>>,
    pub(crate) initial_outputs: Vec<Box<dyn Event>>,
}

impl SimComponentInitializer {
    pub fn new() -> SimComponentInitializer {
        SimComponentInitializer {
            input_events: HashSet::new(),
            output_events: HashSet::new(),
            pending_notifies: Vec::new(),
            pending_transforms: Vec::new(),
            initial_outputs: Vec::new(),
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
            panic!("Components cannot register notifications for Events they are producing! This could cause an infinite loop.")
        }

        self.pending_notifies.push((priority,Box::new(default)))
    }
    
    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim`.
    /// 
    /// ### Arguments
    /// * `handler` - Function to modify the `Event`
    pub fn transform<T: Event>(&mut self, handler: impl FnMut(&mut T) + 'static) {
        self.pending_transforms.push(Box::new(TransformerItem::new(handler)))
    }
    
    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim` with a given priority value.
    /// 
    /// ### Arguments
    /// * `priority` - Transformation order priority for this registration
    /// * `handler` - Function to modify the `Event`
    pub fn transform_prioritized<T: Event>(&mut self, priority: i32, handler: impl FnMut(&mut T) + 'static) {
        self.pending_transforms.push(Box::new(TransformerItem::new_prioritized(handler, priority)))
    }
    
    /// Sets an `Event` as the initial state on the `Sim`
    /// 
    /// ### Arguments
    /// * `event` - `Event` instance to set on initial state
    pub fn set_output<T: Event>(&mut self, initial_value: T) {
        self.initial_outputs.push(Box::new(initial_value))
    }
}

// Unset any listeners & transformers when this object drops
// impl<'a> Drop for SimComponentInitializer<'a> {
//     fn drop(&mut self) {
//         let mut hub = self.hub.borrow_mut();
//         for listener_id in self.listener_ids.iter_mut() {
//             match hub.off(*listener_id) {
//                 Err(err) => panic!(err),
//                 Ok(_) => {}
//             }
//         }

//         for transformer_id in self.transformer_ids.iter_mut() {
//             match hub.unset_transform(*transformer_id) {
//                 Err(err) => panic!(err),
//                 Ok(_) => {}
//             }
//         }
//     }
// }
