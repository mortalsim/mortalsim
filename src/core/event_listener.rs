//! Base EventListener class
//!
//! Provides an Ord wrapper for `Event` handling functions

use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use uuid::Uuid;
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

pub struct EventListener<'a, T: Event> {
    /// Unique identifier for this listener
    listener_id: Uuid,
    /// Container for the Event handling function
    handler: Box<dyn FnMut(&T) + 'a>,
    /// Priority for this listener
    priority: i32
}

impl<'a, T: Event> EventListener<'a, T> {
    /// Creates a new EventListener for the given handler function with
    /// the default priority of 0
    ///
    /// # Arguments
    /// * `handler` - Event handling function
    pub fn new(handler: impl FnMut(&T) + 'a) -> EventListener<'a, T> {
        EventListener {
            listener_id: Uuid::new_v4(),
            handler: Box::new(handler),
            priority: 0
        }
    }
    /// Creates a new EventListener for the given handler function
    /// and priority of execution
    /// 
    /// # Arguments
    /// * `handler`  - Event handling function
    /// * `priority` - determines this listener's priority when Events
    ///                are dispatched. Higher priority listeners are
    ///                executed first.
    pub fn new_prioritized(handler: impl FnMut(&T) + 'a, priority: i32) -> EventListener<'a, T> {
        EventListener {
            listener_id: Uuid::new_v4(),
            handler: Box::new(handler),
            priority: priority
        }
    }
    /// Calls this listener's handler function with the given Event
    ///
    /// # Arguments
    /// * `evt` - Event to dispatch to the handler function
    pub fn handle(&mut self, evt: &dyn Event) {
        match evt.downcast_ref::<T>() {
            Some(typed_evt) => (*self.handler)(typed_evt),
            None => panic!("Something really went wrong here...")
        }
    }
}

// Implement all the traits we need to support Ord
impl<'a, T: Event> PartialEq for EventListener<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl<'a, T: Event> PartialOrd for EventListener<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<'a, T: Event> Eq for EventListener<'a, T> {}

impl<'a, T: Event> Ord for EventListener<'a, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::EventListener;
    use crate::core::event::TestEventA;
    use uom::si::f64::Length;
    use uom::si::length::meter;

    #[test]
    fn test_handle() {
        let val: Cell<Length> = Cell::new(Length::new::<meter>(0.0));
        let mut listener = EventListener::new(|evt: &TestEventA| {
            val.set(evt.len);
        });

        listener.handle(&TestEventA {
            len: Length::new::<meter>(5.0)
        });

        assert_eq!(val.get(), Length::new::<meter>(5.0));
        
        listener.handle(&TestEventA {
            len: Length::new::<meter>(7.0)
        });

        assert_eq!(val.get(), Length::new::<meter>(7.0));
    }

    #[test]
    fn test_ord() {
        let listener1 = EventListener::new_prioritized(|_evt: &TestEventA| {}, 0);
        let listener2 = EventListener::new_prioritized(|_evt: &TestEventA| {}, 5);
        let listener3 = EventListener::new_prioritized(|_evt: &TestEventA| {}, -2);
        let listener4 = EventListener::new_prioritized(|_evt: &TestEventA| {}, 3);

        let mut v = vec!(listener1, listener2, listener3, listener4);

        // Sort in descending order
        v.sort_by(|a,b| b.cmp(a));

        assert_eq!(v[0].priority, 5);
        assert_eq!(v[1].priority, 3);
        assert_eq!(v[2].priority, 0);
        assert_eq!(v[3].priority, -2);
    }

}
