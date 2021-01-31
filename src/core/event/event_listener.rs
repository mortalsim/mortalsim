//! Base EventListener class
//!
//! Provides a wrapper for `Event` handling functions

use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

pub struct EventListener<'a, T: Event> {
    handler: Box<dyn FnMut(&T) + 'a>,
    priority: i32
}

impl<'a, T: Event> EventListener<'a, T>{
    pub fn new(handler: impl FnMut(&T) + 'a) -> EventListener<'a, T> {
        EventListener {
            handler: Box::new(handler),
            priority: 0
        }
    }
    pub fn handle(&mut self, evt: &T) {
        (*self.handler)(evt);
    }
    pub fn get_priority(&self) -> i32 {
        self.priority
    }
    pub fn set_priority(&mut self, priority: i32) {
        self.priority = priority;
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use super::EventListener;
    use crate::core::event::TestEventA;

    #[test]
    fn instantiate() {
        let _listener = EventListener::new(|_evt: &TestEventA| {});
    }
    
    #[test]
    fn test_handle() {
        let val: Cell<i32> = Cell::new(0);
        let mut listener = EventListener::new(|evt: &TestEventA| {
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
