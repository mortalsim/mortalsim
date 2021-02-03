//! Base EventListener class
//!
//! Provides a wrapper for `Event` handling functions

use crate::core::id_gen::{IdType, IdGenerator};
use crate::core::event::Event;
use crate::core::event::EventHandler;

pub trait EventTransformer<T: Event> {
    fn transform(&mut self, evt: &mut T);
    fn get_priority(&self) -> i32;
    fn set_priority(&mut self, priority: i32);
}

pub struct BasicTransformer<'a, T: Event> {
    handler: Box<dyn FnMut(&mut T) + 'a>,
    priority: i32
}

impl<'a, T: Event> BasicTransformer<'a, T> {
    pub fn new(handler: impl FnMut(&mut T) + 'a) -> BasicTransformer<'a, T> {
        BasicTransformer {
            handler: Box::new(handler),
            priority: 0
        }
    }
}

impl<'a, T: Event> EventTransformer<T> for BasicTransformer<'a, T> {
    fn transform(&mut self, evt: &mut T) {
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
    use super::BasicTransformer;
    use super::EventTransformer;
    use crate::core::event::TestEventA;

    #[test]
    fn instantiate() {
        let _listener = BasicTransformer::new(|_evt: &mut TestEventA| {});
    }
    
    #[test]
    fn test_handle() {
        let mut listener = BasicTransformer::new(|evt: &mut TestEventA| {
            evt.value = 10;
        });

        let mut evt = TestEventA {
            value: 5
        };

        listener.transform(&mut evt);
        
        assert_eq!(evt.value, 10);
    }
}
