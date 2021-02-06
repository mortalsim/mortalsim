//! Internal pub/sub interface for Event objects
//!
//! The EventHub handles Event dispatch to their corresponding transformers
//! and listeners based on their TypeId

use std::collections::HashMap;
use core::any::TypeId;
use uuid::Uuid;
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event_listener::{EventListener, ListenerItem};
use crate::core::event_transformer::{EventTransformer, TransformerItem};
use crate::core::event::Event;

pub struct EventHub {
    hub_id: Uuid,
    id_gen: IdGenerator,
    event_listeners: HashMap<TypeId, Vec<(IdType, Box<dyn EventListener>)>>,
    event_transformers: HashMap<TypeId, Vec<(IdType, Box<dyn EventTransformer>)>>,
    generic_event_listeners: Vec<(IdType, Box<dyn FnMut(&dyn Event)>)>,
}

impl EventHub {
    fn new() -> EventHub {
        EventHub {
            hub_id: Uuid::new_v4(),
            id_gen: IdGenerator::new(),
            event_listeners: HashMap::new(),
            event_transformers: HashMap::new(),
            generic_event_listeners: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EventHub;

    #[test]
    fn instantiate() {
        let _hub = EventHub::new();
    }
}