//! Internal pub/sub interface for Event objects
//!
//! This submodule provides the heart of the event simulation. The EventHub
//! handles Event dispatch to their corresponding transformers and listeners
//! based on their TypeId

use crate::core::id_gen::{IdType, IdGenerator};

pub struct EventHub {
    id_gen: IdGenerator
    // TODO
}

impl EventHub {
    fn new() -> EventHub {
        EventHub {
            id_gen: IdGenerator::new()
        }
    }
    // TODO
}

#[cfg(test)]
mod tests {
    use super::EventHub;

    #[test]
    fn instantiate() {
        let hub = EventHub::new();
    }
}