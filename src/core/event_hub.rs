//! Internal pub/sub interface for Event objects
//!
//! The EventHub handles Event dispatch to their corresponding transformers
//! and listeners based on their TypeId

use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use core::any::TypeId;
use uuid::Uuid;
use anyhow::Result;
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event_listener::{EventListener, ListenerItem, GenericListener};
use crate::core::event_transformer::{EventTransformer, TransformerItem};
use crate::core::event::Event;

/// Internal error struct when an invalid listener or transformer id is
/// provided to an EventHub
pub struct InvalidIdError {
    /// Which EventHub object the erroneous id was given to
    eventhub_id: Uuid,
    /// The duplicate id which was returned
    bad_id: IdType
}

impl Error for InvalidIdError {}

impl fmt::Display for InvalidIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ID {} passed to EventHub {}", self.bad_id, self.eventhub_id)?;
        Ok(())
    }
}
impl fmt::Debug for InvalidIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid ID {} passed to EventHub {}, file: {}, line: {}",
            self.bad_id, self.eventhub_id, file!(), line!())?;
        Ok(())
    }
}

pub struct EventHub<'a> {
    /// Id for this EventHub
    hub_id: Uuid,
    /// Map of listeners for particular Events
    event_listeners: HashMap<TypeId, Vec<Box<dyn EventListener + 'a>>>,
    /// Map of transformers for particular Events
    event_transformers: HashMap<TypeId, Vec<Box<dyn EventTransformer + 'a>>>,
    /// Listeners for any Event, regardless of Event type
    generic_event_listeners: Vec<Box<dyn EventListener + 'a>>,
}

impl<'a> EventHub<'a> {
    pub fn new() -> EventHub<'a> {
        EventHub {
            hub_id: Uuid::new_v4(),
            event_listeners: HashMap::new(),
            event_transformers: HashMap::new(),
            generic_event_listeners: Vec::new(),
        }
    }

    pub fn emit<T: Event>(&mut self, mut evt: T) {
        let type_key = TypeId::of::<T>();

        // Call each transformer with the event
        match self.event_transformers.get_mut(&type_key) {
            None => {}, // No transformers = nothing to do
            Some(transformers) => {
                log::debug!("Triggering {} transformers for EventHub {}", transformers.len(), self.hub_id);
                for transformer in transformers {
                    transformer.transform(&mut evt);
                }
            }
        }
        
        // Call each generic listener with the event
        log::debug!("Triggering {} generic listeners for EventHub {}", self.generic_event_listeners.len(), self.hub_id);
        for listener in &mut self.generic_event_listeners {
            listener.handle(&evt);
        }

        log::debug!("All typed listener lengths:");
        for (type_id, listeners) in &self.event_listeners {
            log::debug!("{:?} : {:?}", type_id, listeners.len());
        }
        
        // Call each typed listener with the event
        match self.event_listeners.get_mut(&type_key) {
            None => {}, // No listeners = nothing to do
            Some(listeners) => {
                log::debug!("Triggering {} transformers for EventHub {}", listeners.len(), self.hub_id);
                for listener in listeners {
                    listener.handle(&evt);
                }
            }
        }
    }

    pub fn on_any(&mut self, handler: impl FnMut(&dyn Event) + 'a) -> IdType {
        self.on_any_impl(Box::new(GenericListener::new(handler)))
    }
    
    pub fn on_any_prioritized(&mut self, priority: i32, handler: impl FnMut(&dyn Event) + 'a) -> IdType {
        self.on_any_impl(Box::new(GenericListener::new_prioritized(handler, priority)))
    }
    
    fn on_any_impl(&mut self, listener: Box<dyn EventListener + 'a>) -> IdType {

        let listener_id = listener.listener_id();

        // insert the listener into the generic_event_listeners at the appropriate location
        match self.generic_event_listeners.binary_search(&listener) {
            Ok(_) => panic!("Duplicate Generic Listeners with id {}", listener.listener_id()),
            Err(pos) => {
                self.generic_event_listeners.insert(pos, listener);
            }
        }

        // Return the listener id to the caller
        listener_id
    }

    pub fn off_any(&mut self, listener_id: IdType) -> Result<()> {
        match self.generic_event_listeners.iter().position(|l| l.listener_id() == listener_id) {
            Some(pos) => {
                self.generic_event_listeners.remove(pos);
                Ok(())
            }
            None => Err(anyhow::Error::new(InvalidIdError {eventhub_id: self.hub_id, bad_id: listener_id}))
        }
    }

    pub fn on<T: Event>(&mut self, handler: impl FnMut(&T) + 'a) -> IdType {
        self.on_impl::<T>(Box::new(ListenerItem::new(handler)))
    }
    
    pub fn on_prioritized<T: Event>(&mut self, priority: i32, handler: impl FnMut(&T) + 'a) -> IdType {
        self.on_impl::<T>(Box::new(ListenerItem::new_prioritized(handler, priority)))
    }
    
    fn on_impl<T: Event>(&mut self, listener: Box<dyn EventListener + 'a>) -> IdType {
        let type_key = TypeId::of::<T>();

        // Keep a reference to the listener's id so we can return it
        let listener_id = listener.listener_id();

        log::debug!("Adding new listener for key {:?}", type_key);

        // insert the listener into the listeners array
        match self.event_listeners.get(&type_key) {
            Some(listeners) => {
                match listeners.binary_search(&listener) {
                    Ok(_) => panic!("Duplicate Listener id {}", listener.listener_id()),
                    Err(pos) => {
                        self.event_listeners.get_mut(&type_key).unwrap().insert(pos, listener);
                    }
                }
            },
            None => {
                self.event_listeners.insert(type_key, vec!(listener));
            }
        }

        // Return the listener id to the caller
        listener_id
    }

    pub fn off<T: Event>(&mut self, listener_id: IdType) -> Result<()> {
        let type_key = TypeId::of::<T>();

        match self.event_listeners.get_mut(&type_key) {
            Some(listeners) => {
                match listeners.iter().position(|l| l.listener_id() == listener_id) {
                    Some(pos) => {
                        listeners.remove(pos);
                        Ok(())
                    },
                    None => Err(anyhow::Error::new(InvalidIdError {eventhub_id: self.hub_id, bad_id: listener_id}))
                }
            }
            None => Err(anyhow::Error::new(InvalidIdError {eventhub_id: self.hub_id, bad_id: listener_id}))
        }
    }

    pub fn transform<T: Event>(&mut self, handler: impl FnMut(&mut T) + 'a) -> IdType {
        self.transform_impl::<T>(Box::new(TransformerItem::new(handler)))
    }
    
    pub fn transform_prioritized<T: Event>(&mut self, priority: i32, handler: impl FnMut(&mut T) + 'a) -> IdType {
        self.transform_impl::<T>(Box::new(TransformerItem::new_prioritized(handler, priority)))
    }

    fn transform_impl<T: Event>(&mut self, transformer: Box<dyn EventTransformer + 'a>) -> IdType {
        let type_key = TypeId::of::<T>();

        // Keep a reference to the transformer's id so we can return it
        let transformer_id = transformer.transformer_id();

        // insert the transformer into the transformers array
        match self.event_transformers.get(&type_key) {
            Some(transformers) => {
                match transformers.binary_search(&transformer) {
                    Ok(_) => panic!("Duplicate Transformer id {}", transformer.transformer_id()),
                    Err(pos) => {
                        self.event_transformers.get_mut(&type_key).unwrap().insert(pos, transformer);
                    }
                }
            },
            None => {
                self.event_transformers.insert(type_key, vec!(transformer));
            }
        }

        // Return the transformer id to the caller
        transformer_id
    }

    pub fn unset_transform<T: Event>(&mut self, transformer_id: IdType) -> Result<()> {
        let type_key = TypeId::of::<T>();

        match self.event_transformers.get_mut(&type_key) {
            Some(transformers) => {
                match transformers.iter().position(|l| l.transformer_id() == transformer_id) {
                    Some(pos) => {
                        transformers.remove(pos);
                        Ok(())
                    },
                    None => Err(anyhow::Error::new(InvalidIdError {eventhub_id: self.hub_id, bad_id: transformer_id}))
                }
            }
            None => Err(anyhow::Error::new(InvalidIdError {eventhub_id: self.hub_id, bad_id: transformer_id}))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use uom::si::f64::Length;
    use uom::si::f64::AmountOfSubstance;
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;
    use simple_logger::SimpleLogger;
    use super::EventHub;
    use crate::core::event::Event;
    use crate::core::event::test::TestEventA;
    use crate::core::event::test::TestEventB;

    #[test]
    fn test_hub() {
        crate::init_test();

        let any_count = Cell::new(0);
        let a_count = Cell::new(0);
        let on_val_a = Cell::new(Length::new::<meter>(0.0));
        let b_count = Cell::new(0);
        let on_val_b = Cell::new(AmountOfSubstance::new::<mole>(0.0));

        let mut hub = EventHub::new();

        // Attach a handler for A Events
        hub.on(|evt: &TestEventA| {
            on_val_a.set(evt.len);
            a_count.set(a_count.get() + 1);
        });

        // Emit an event and it should get passed to the appropriate handler
        hub.emit(TestEventA::new(Length::new::<meter>(1.0)));
        assert_eq!(on_val_a.get(), Length::new::<meter>(1.0));
        assert_eq!(a_count.get(), 1);
        
        // Attach a handler for any Event
        hub.on_any(|_evt: &dyn Event| {
            any_count.set(any_count.get() + 1);
        });

        // Emitting an A event should now cause both to be called
        hub.emit(TestEventA::new(Length::new::<meter>(2.0)));
        assert_eq!(on_val_a.get(), Length::new::<meter>(2.0));
        assert_eq!(a_count.get(), 2);
        assert_eq!(any_count.get(), 1);
        
        // Attach a handler for B Events
        hub.on(|evt: &TestEventB| {
            on_val_b.set(evt.amt);
            b_count.set(b_count.get() + 1);
        });

        // Emitting a B event should call the B and any handlers only
        hub.emit(TestEventB::new(AmountOfSubstance::new::<mole>(1.0)));
        assert_eq!(on_val_b.get(), AmountOfSubstance::new::<mole>(1.0));
        assert_eq!(a_count.get(), 2);
        assert_eq!(b_count.get(), 1);
        assert_eq!(any_count.get(), 2);

        // Attach a transformer for A that overrides any value
        hub.transform(|evt: &mut TestEventA| {
            evt.len = Length::new::<meter>(10.0);
        });

        // It should be set to that value now whenever A Events are emitted
        hub.emit(TestEventA::new(Length::new::<meter>(3.0)));
        assert_eq!(on_val_a.get(), Length::new::<meter>(10.0));
        hub.emit(TestEventA::new(Length::new::<meter>(5.0)));
        assert_eq!(on_val_a.get(), Length::new::<meter>(10.0));
    }

    #[test]
    fn test_prioritized() {
        crate::init_test();
        // TODO
    }
}