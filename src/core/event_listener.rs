//! Base ListenerItem class
//!
//! Provides an Ord wrapper for `Event` handling functions

use std::cmp;
use std::sync::Mutex;
use uuid::Uuid;
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

lazy_static! {
    static ref ID_GEN: Mutex<IdGenerator> = Mutex::new(IdGenerator::new());
}

pub trait EventListener {
    /// Calls this listener's handler function with the given Event
    ///
    /// # Arguments
    /// * `evt` - Event to dispatch to the handler function
    fn handle(&mut self, evt: &dyn Event);

    /// Retrieves the priority value for this listener
    fn priority(&self) -> i32;
    
    /// Retrieves the id for this listener
    fn listener_id(&self) -> IdType;
}

// Implement all the traits we need to support Ord
impl<'a> PartialEq for dyn EventListener + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.listener_id() == other.listener_id()
    }
}

impl<'a> PartialOrd for dyn EventListener + 'a {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        other.priority().partial_cmp(&self.priority())
    }
}

impl<'a> Eq for dyn EventListener + 'a {}

impl<'a> Ord for dyn EventListener + 'a {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        other.priority().cmp(&self.priority())
    }
}

pub struct GenericListener<'a> {
    /// Unique identifier for this listener
    listener_id: IdType,
    /// Container for the Event handling function
    handler: Box<dyn FnMut(&dyn Event) + 'a>,
    /// Priority for this listener
    priority: i32
}

impl<'a> GenericListener<'a> {
    /// Creates a new GenericListener for the given handler function with
    /// the default priority of 0
    ///
    /// # Arguments
    /// * `id` - Identifier for this listener
    pub fn new(handler: impl FnMut(&dyn Event) + 'a) -> GenericListener<'a> {
        GenericListener {
            listener_id: ID_GEN.lock().unwrap().get_id(),
            priority: 0,
            handler: Box::new(handler),
        }
    }
    /// Creates a new GenericListener for the given handler function
    /// and priority of execution
    /// 
    /// # Arguments
    /// * `handler`  - Event handling function
    /// * `priority` - determines this listener's priority when Events
    ///                are dispatched. Higher priority listeners are
    ///                executed first.
    pub fn new_prioritized(handler: impl FnMut(&dyn Event) + 'a, priority: i32) -> GenericListener<'a> {
        GenericListener {
            listener_id: ID_GEN.lock().unwrap().get_id(),
            handler: Box::new(handler),
            priority: priority
        }
    }
}

impl<'a> Drop for GenericListener<'a> {
    fn drop(&mut self) {
        // Return ids back to the pool when listeners are dropped
        ID_GEN.lock().unwrap().return_id(self.listener_id).unwrap();
    }
}

impl<'a> EventListener for GenericListener<'a> {
    fn handle(&mut self, evt: &dyn Event) {
        log::debug!("Executing generic event listener {} with Event {}", self.listener_id, evt.event_id());
        (*self.handler)(evt);
    }

    fn priority(&self) -> i32 {
        self.priority
    }
    
    fn listener_id(&self) -> IdType {
        self.listener_id
    }
}

pub struct ListenerItem<'a, T: Event> {
    /// Unique identifier for this listener
    listener_id: IdType,
    /// Container for the Event handling function
    handler: Box<dyn FnMut(&T) + 'a>,
    /// Priority for this listener
    priority: i32
}

impl<'a, T: Event> ListenerItem<'a, T> {
    /// Creates a new ListenerItem for the given handler function with
    /// the default priority of 0
    ///
    /// # Arguments
    /// * `id` - Identifier for this listener
    pub fn new(handler: impl FnMut(&T) + 'a) -> ListenerItem<'a, T> {
        ListenerItem {
            listener_id: ID_GEN.lock().unwrap().get_id(),
            priority: 0,
            handler: Box::new(handler),
        }
    }
    /// Creates a new ListenerItem for the given handler function
    /// and priority of execution
    /// 
    /// # Arguments
    /// * `handler`  - Event handling function
    /// * `priority` - determines this listener's priority when Events
    ///                are dispatched. Higher priority listeners are
    ///                executed first.
    pub fn new_prioritized(handler: impl FnMut(&T) + 'a, priority: i32) -> ListenerItem<'a, T> {
        ListenerItem {
            listener_id: ID_GEN.lock().unwrap().get_id(),
            handler: Box::new(handler),
            priority: priority
        }
    }
}

impl<'a, T: Event> Drop for ListenerItem<'a, T> {
    fn drop(&mut self) {
        // Return ids back to the pool when listeners are dropped
        ID_GEN.lock().unwrap().return_id(self.listener_id).unwrap();
    }
}

impl<'a, T: Event> EventListener for ListenerItem<'a, T> {
    fn handle(&mut self, evt: &dyn Event) {
        log::debug!("Executing event listener {} with Event {}", self.listener_id, evt.event_id());
        match evt.downcast_ref::<T>() {
            Some(typed_evt) => (*self.handler)(typed_evt),
            None => panic!("Ahhh! Listener {} is on fire!!!", self.listener_id)
        }
    }

    fn priority(&self) -> i32 {
        self.priority
    }
    
    fn listener_id(&self) -> IdType {
        self.listener_id
    }
}


#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::ListenerItem;
    use super::EventListener;
    use crate::core::event::test::TestEventA;
    use uom::si::f64::Length;
    use uom::si::length::meter;

    #[test]
    fn test_handle() {
        let val: Cell<Length> = Cell::new(Length::new::<meter>(0.0));
        let mut listener = ListenerItem::new(|evt: &TestEventA| {
            val.set(evt.len);
        });

        listener.handle(&TestEventA::new(Length::new::<meter>(5.0)));
        assert_eq!(val.get(), Length::new::<meter>(5.0));
        
        listener.handle(&TestEventA::new(Length::new::<meter>(7.0)));
        assert_eq!(val.get(), Length::new::<meter>(7.0));
    }

    #[test]
    fn test_ord() {
        let listener1 = ListenerItem::new_prioritized(|_evt: &TestEventA| {}, 0);
        let listener2 = ListenerItem::new_prioritized(|_evt: &TestEventA| {}, 5);
        let listener3 = ListenerItem::new_prioritized(|_evt: &TestEventA| {}, -2);
        let listener4 = ListenerItem::new_prioritized(|_evt: &TestEventA| {}, 3);

        let mut v = Vec::<Box<dyn EventListener>>::new();

        v.push(Box::new(listener1));
        v.push(Box::new(listener2));
        v.push(Box::new(listener3));
        v.push(Box::new(listener4));

        v.sort();

        assert_eq!(v[0].priority(), 5);
        assert_eq!(v[1].priority(), 3);
        assert_eq!(v[2].priority(), 0);
        assert_eq!(v[3].priority(), -2);
    }

}
