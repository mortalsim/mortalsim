//! Base EventListener class
//!
//! Provides a wrapper for `Event` handling functions

use std::marker::Sized;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use mopa::Any;
use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

pub trait EventListener {
    fn handle(&mut self, evt: &dyn Event);
    fn get_priority(&self) -> i32;
    fn set_priority(&mut self, priority: i32);
}

impl PartialEq for dyn EventListener {
    fn eq(&self, other: &Self) -> bool {
        self.get_priority() == other.get_priority()
    }
}

impl PartialOrd for dyn EventListener {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_priority().partial_cmp(&other.get_priority())
    }
}

impl Eq for dyn EventListener {}

impl Ord for dyn EventListener {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_priority().cmp(&other.get_priority())
    }
}

pub struct BasicListener<'a, T: Event> {
    handler: Box<dyn FnMut(&T) + 'a>,
    priority: i32
}

impl<'a, T: Event> BasicListener<'a, T> {
    pub fn new(handler: impl FnMut(&T) + 'a) -> BasicListener<'a, T> {
        BasicListener {
            handler: Box::new(handler),
            priority: 0
        }
    }
}

impl<'a, T: Event> EventListener for BasicListener<'a, T> {
    fn handle(&mut self, evt: &dyn Event) {
        match evt.downcast_ref::<T>() {
            Some(typed_evt) => (*self.handler)(typed_evt),
            None => panic!("Something really went wrong here...")
        }
    }
    fn get_priority(&self) -> i32 {
        self.priority
    }
    fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::BinaryHeap;
    use super::BasicListener;
    use super::EventListener;
    use crate::core::event::TestEventA;
    use uom::si::f64::Length;
    use uom::si::length::meter;

    #[test]
    fn instantiate() {
        let _listener = BasicListener::new(|_evt: &TestEventA| {});
    }
    
    #[test]
    fn test_handle() {
        let val: Cell<Length> = Cell::new(Length::new::<meter>(0.0));
        let mut listener = BasicListener::new(|evt: &TestEventA| {
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

}
