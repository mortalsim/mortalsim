//! Base EventTransformer class
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

pub struct EventTransformer<'a, T: Event> {
    /// Unique identifier for this listener
    transformer_id: Uuid,
    /// Container for the Event transforming function
    handler: Box<dyn FnMut(&mut T) + 'a>,
    /// Priority for this transformer
    priority: i32
}

impl<'a, T: Event> EventTransformer<'a, T> {
    /// Creates a new EventTransformer for the given handler with
    /// the default priority of 0
    ///
    /// # Arguments
    /// * `handler` - Event transforming function
    pub fn new(handler: impl FnMut(&mut T) + 'a) -> EventTransformer<'a, T> {
        EventTransformer {
            transformer_id: Uuid::new_v4(),
            handler: Box::new(handler),
            priority: 0
        }
    }
    /// Creates a new EventTransformer for the given handler and
    /// priority of execution
    ///
    /// # Arguments
    /// * `handler` - Event transforming function
    /// * `priority` - Event transforming function
    /// * `priority` - determines this transformer's priority when Events
    ///                are dispatched. Higher priority transformers are
    ///                executed first.
    pub fn new_prioritized(handler: impl FnMut(&mut T) + 'a, priority: i32) -> EventTransformer<'a, T> {
        EventTransformer {
            transformer_id: Uuid::new_v4(),
            handler: Box::new(handler),
            priority: priority
        }
    }
    /// Calls this transformer's handler function with the given Event
    ///
    /// # Arguments
    /// * `evt` - Event to dispatch to the handler function
    pub fn transform(&mut self, evt: &mut T) {
        (*self.handler)(evt);
    }
}

// Implement all the traits we need to support Ord
impl<'a, T: Event> PartialEq for EventTransformer<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<'a, T: Event> PartialOrd for EventTransformer<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<'a, T: Event> Eq for EventTransformer<'a, T> {}

impl<'a, T: Event> Ord for EventTransformer<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::EventTransformer;
    use crate::core::event::TestEventA;
    use uom::si::f64::Length;
    use uom::si::length::meter;

    #[test]
    fn test_handle() {
        let mut listener = EventTransformer::new(|evt: &mut TestEventA| {
            evt.len = Length::new::<meter>(10.0);
        });

        let mut evt = TestEventA {
            len: Length::new::<meter>(5.0)
        };

        listener.transform(&mut evt);
        
        assert_eq!(evt.len, Length::new::<meter>(10.0));
    }
    
    #[test]
    fn test_ord() {
        let listener1 = EventTransformer::new_prioritized(|_evt: &mut TestEventA| {}, 0);
        let listener2 = EventTransformer::new_prioritized(|_evt: &mut TestEventA| {}, 5);
        let listener3 = EventTransformer::new_prioritized(|_evt: &mut TestEventA| {}, -2);
        let listener4 = EventTransformer::new_prioritized(|_evt: &mut TestEventA| {}, 3);

        let mut v = vec!(listener1, listener2, listener3, listener4);

        // Sort in descending order
        v.sort_by(|a,b| b.cmp(a));

        assert_eq!(v[0].priority, 5);
        assert_eq!(v[1].priority, 3);
        assert_eq!(v[2].priority, 0);
        assert_eq!(v[3].priority, -2);
    }
}
