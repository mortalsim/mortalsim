//! Base EventListener class
//!
//! Provides a wrapper for `Event` handling functions

use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

pub trait EventListener<T: Event> {
    fn handle(&mut self, evt: &T);
    fn get_priority(&self) -> i32;
    fn set_priority(&mut self, priority: i32);
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

impl<'a, T: Event> EventListener<T> for BasicListener<'a, T> {
    fn handle(&mut self, evt: &T) {
        (*self.handler)(evt);
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
    use super::BasicListener;
    use super::EventListener;
    use crate::core::event::TestEventA;

    #[test]
    fn instantiate() {
        let _listener = BasicListener::new(|_evt: &TestEventA| {});
    }
    
    #[test]
    fn test_handle() {
        let val: Cell<i32> = Cell::new(0);
        let mut listener = BasicListener::new(|evt: &TestEventA| {
            val.set(evt.value);
        });

        listener.handle(&TestEventA {
            value: 5
        });

        assert_eq!(val.get(), 5);
        
        listener.handle(&TestEventA {
            value: 7
        });

        assert_eq!(val.get(), 7);
    }
}
