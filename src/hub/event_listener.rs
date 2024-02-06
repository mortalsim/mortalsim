//! Base ListenerItem class
//!
//! Provides an Ord wrapper for `Event` handling functions

use crate::event::Event;
use crate::util::id_gen::{IdGenerator, IdType};
use std::cmp;
use std::fmt;
use std::sync::{Arc, MutexGuard, OnceLock};
use std::sync::Mutex;

static ID_GEN: OnceLock<Mutex<IdGenerator>> = OnceLock::new();

pub trait EventListener: Send {
    /// Calls this listener's handler function with the given Event
    ///
    /// ### Arguments
    /// * `evt` - Event to dispatch to the handler function
    fn handle(&mut self, evt: Box<dyn Event>);

    /// Retrieves the priority value for this listener
    fn priority(&self) -> i32;

    /// Retrieves the id for this listener
    fn listener_id(&self) -> IdType;
}

impl<'a> fmt::Debug for dyn EventListener + 'a {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EventListener<{:?}> {{ priority: {:?} }}",
            self.listener_id(),
            self.priority()
        )
    }
}

// Implement all the traits we need to support Ord
impl<'a> PartialEq for dyn EventListener + 'a {
    fn eq(&self, other: &Self) -> bool {
        self.listener_id() == other.listener_id()
    }
}

impl<'a> PartialOrd for dyn EventListener + 'a {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        if other.priority() == self.priority() {
            if self.eq(other) {
                Some(cmp::Ordering::Equal)
            } else {
                self.listener_id().partial_cmp(&other.listener_id())
            }
        } else {
            other.priority().partial_cmp(&self.priority())
        }
    }
}

impl<'a> Eq for dyn EventListener + 'a {}

impl<'a> Ord for dyn EventListener + 'a {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if other.priority() == self.priority() {
            if self.eq(other) {
                cmp::Ordering::Equal
            } else {
                self.listener_id().cmp(&other.listener_id())
            }
        } else {
            other.priority().cmp(&self.priority())
        }
    }
}

pub struct GenericListener<'a> {
    /// Unique identifier for this listener
    listener_id: IdType,
    /// Container for the Event handling function
    handler: Box<dyn FnMut(Box<dyn Event>) + Send + 'a>,
    /// Priority for this listener
    priority: i32,
}

impl<'a> GenericListener<'a> {
    fn id_gen() -> MutexGuard<'static, IdGenerator> {
        ID_GEN.get_or_init(|| {
            Mutex::new(IdGenerator::new())
        }).lock().unwrap()
    }

    /// Creates a new GenericListener for the given handler function with
    /// the default priority of 0
    ///
    /// ### Arguments
    /// * `id` - Identifier for this listener
    pub fn new(handler: impl FnMut(Box<dyn Event>) + Send + 'a) -> GenericListener<'a> {
        GenericListener {
            listener_id: Self::id_gen().get_id(),
            priority: 0,
            handler: Box::new(handler),
        }
    }
    /// Creates a new GenericListener for the given handler function
    /// and priority of execution
    ///
    /// ### Arguments
    /// * `handler`  - Event handling function
    /// * `priority` - determines this listener's priority when Events
    ///                are dispatched. Higher priority listeners are
    ///                executed first.
    pub fn new_prioritized(
        handler: impl FnMut(Box<dyn Event>) + Send + 'a,
        priority: i32,
    ) -> GenericListener<'a> {
        GenericListener {
            listener_id: Self::id_gen().get_id(),
            handler: Box::new(handler),
            priority: priority,
        }
    }
}

impl<'a> Drop for GenericListener<'a> {
    fn drop(&mut self) {
        // Return ids back to the pool when listeners are dropped
        Self::id_gen().return_id(self.listener_id).unwrap();
    }
}

impl<'a> EventListener for GenericListener<'a> {
    fn handle(&mut self, evt: Box<dyn Event>) {
        log::debug!(
            "Executing generic event listener {} with Event {}",
            self.listener_id,
            evt.event_name()
        );
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
    handler: Box<dyn FnMut(Box<T>) + Send + 'a>,
    /// Priority for this listener
    priority: i32,
}

impl<'a, T: Event> ListenerItem<'a, T> {
    fn id_gen() -> MutexGuard<'static, IdGenerator> {
        ID_GEN.get_or_init(|| {
            Mutex::new(IdGenerator::new())
        }).lock().unwrap()
    }

    /// Creates a new ListenerItem for the given handler function with
    /// the default priority of 0
    ///
    /// ### Arguments
    /// * `id` - Identifier for this listener
    pub fn new(handler: impl FnMut(Box<T>) + Send + 'a) -> ListenerItem<'a, T> {
        ListenerItem {
            listener_id: Self::id_gen().get_id(),
            priority: 0,
            handler: Box::new(handler),
        }
    }
    /// Creates a new ListenerItem for the given handler function
    /// and priority of execution
    ///
    /// ### Arguments
    /// * `priority` - determines this listener's priority when Events
    ///                are dispatched. Higher priority listeners are
    ///                executed first.
    /// * `handler`  - Event handling function
    pub fn new_prioritized(priority: i32, handler: impl FnMut(Box<T>) + Send + 'a) -> ListenerItem<'a, T> {
        ListenerItem {
            listener_id: Self::id_gen().get_id(),
            handler: Box::new(handler),
            priority: priority,
        }
    }
}

impl<'a, T: Event> Drop for ListenerItem<'a, T> {
    fn drop(&mut self) {
        // Return ids back to the pool when listeners are dropped
        Self::id_gen().return_id(self.listener_id).unwrap();
    }
}

impl<'a, T: Event> EventListener for ListenerItem<'a, T> {
    fn handle(&mut self, evt: Box<dyn Event>) {
        log::debug!(
            "Executing event listener {} with Event {}",
            self.listener_id,
            evt.event_name()
        );

        match evt.downcast::<T>() {
            Ok(typed_evt) => (*self.handler)(typed_evt),
            Err(_) => panic!("Ahhh! Listener {} is melting!!!", self.listener_id),
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
    use super::EventListener;
    use super::ListenerItem;
    use crate::event::test::TestEventA;
    use crate::units::base::Distance;
    use std::borrow::Borrow;
    use std::borrow::BorrowMut;
    use std::cell::Cell;
    use std::sync::Arc;
    use std::sync::RwLock;

    #[test]
    fn test_handle() {
        let val: RwLock<Distance<f64>> = RwLock::new(Distance::from_m(0.0));
        let mut listener = ListenerItem::new(|evt: Box<TestEventA>| {
            *val.write().unwrap() = evt.len;
        });

        listener.handle(Box::new(TestEventA::new(Distance::from_m(5.0))));
        assert_eq!(*val.read().unwrap(), Distance::from_m(5.0));

        listener.handle(Box::new(TestEventA::new(Distance::from_m(7.0))));
        assert_eq!(*val.read().unwrap(), Distance::from_m(7.0));
    }

    #[test]
    fn test_ord() {
        let listener1 = ListenerItem::new_prioritized(0, |_evt: Box<TestEventA>| {});
        let listener2 = ListenerItem::new_prioritized(5, |_evt: Box<TestEventA>| {});
        let listener3 = ListenerItem::new_prioritized(-2, |_evt: Box<TestEventA>| {});
        let listener4 = ListenerItem::new_prioritized(3, |_evt: Box<TestEventA>| {});

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
