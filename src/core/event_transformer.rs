//! Base TransformerItem class
//!
//! Provides an Ord wrapper for `Event` transforming functions

use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use uuid::Uuid;
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

pub trait EventTransformer {
    /// Calls this transformer's handler function with the given Event
    ///
    /// # Arguments
    /// * `evt` - Event to dispatch to the handler function
    fn transform(&mut self, evt: &mut dyn Event);

    /// Retrieves the priority value for this transformer
    fn priority(&self) -> i32;
}

// Implement all the traits we need to support Ord
impl PartialEq for dyn EventTransformer {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

impl PartialOrd for dyn EventTransformer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority().partial_cmp(&other.priority())
    }
}

impl Eq for dyn EventTransformer {}

impl Ord for dyn EventTransformer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

pub struct TransformerItem<'a, T: Event> {
    /// Unique identifier for this listener
    transformer_id: Uuid,
    /// Container for the Event transforming function
    handler: Box<dyn FnMut(&mut T) + 'a>,
    /// Priority for this transformer
    priority: i32
}

impl<'a, T: Event> TransformerItem<'a, T> {
    /// Creates a new TransformerItem for the given handler with
    /// the default priority of 0
    ///
    /// # Arguments
    /// * `handler` - Event transforming function
    pub fn new(handler: impl FnMut(&mut T) + 'a) -> TransformerItem<'a, T> {
        TransformerItem {
            transformer_id: Uuid::new_v4(),
            handler: Box::new(handler),
            priority: 0
        }
    }
    /// Creates a new TransformerItem for the given handler and
    /// priority of execution
    ///
    /// # Arguments
    /// * `handler` - Event transforming function
    /// * `priority` - Event transforming function
    /// * `priority` - determines this transformer's priority when Events
    ///                are dispatched. Higher priority transformers are
    ///                executed first.
    pub fn new_prioritized(handler: impl FnMut(&mut T) + 'a, priority: i32) -> TransformerItem<'a, T> {
        TransformerItem {
            transformer_id: Uuid::new_v4(),
            handler: Box::new(handler),
            priority: priority
        }
    }
}
    
impl<'a, T: Event> EventTransformer for TransformerItem<'a, T> {
    fn transform(&mut self, evt: &mut dyn Event) {
        match evt.downcast_mut::<T>() {
            Some(typed_evt) => (*self.handler)(typed_evt),
            None => panic!("Ahhh! Transformer {} is on fire!!!", self.transformer_id)
        }
    }
    
    fn priority(&self) -> i32 {
        self.priority
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::TransformerItem;
    use super::EventTransformer;
    use crate::core::event::test::TestEventA;
    use uom::si::f64::Length;
    use uom::si::length::meter;

    #[test]
    fn test_handle() {
        let mut listener = TransformerItem::new(|evt: &mut TestEventA| {
            evt.len = Length::new::<meter>(10.0);
        });

        let mut evt = TestEventA::new(Length::new::<meter>(5.0));

        listener.transform(&mut evt);
        
        assert_eq!(evt.len, Length::new::<meter>(10.0));
    }
    
    #[test]
    fn test_ord() {
        let transformer1 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, 0);
        let transformer2 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, 5);
        let transformer3 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, -2);
        let transformer4 = TransformerItem::new_prioritized(|_evt: &mut TestEventA| {}, 3);

        let mut v = Vec::<Box<dyn EventTransformer>>::new();

        v.push(Box::new(transformer1));
        v.push(Box::new(transformer2));
        v.push(Box::new(transformer3));
        v.push(Box::new(transformer4));

        v.sort();

        assert_eq!(v[0].priority(), -2);
        assert_eq!(v[1].priority(), 0);
        assert_eq!(v[2].priority(), 3);
        assert_eq!(v[3].priority(), 5);
    }
}
