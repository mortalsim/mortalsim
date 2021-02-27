use std::collections::HashMap;
use std::any::TypeId;
use crate::event::Event;
use anyhow::{Result, Error};

pub struct SimState {
    state: HashMap<TypeId, Box<dyn Event>>
}

impl SimState {
    /// Creates a new `SimState` object
    pub fn new() -> SimState {
        SimState {
            state: HashMap::new()
        }
    }

    /// Retrieves the current `Event` of a given type in this state
    /// 
    /// returns an `Event` or `None` if no `Event` of this type has been set
    pub fn get_state<T: Event>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        match self.state.get(&type_id) {
            None => None,
            Some(box_val) => {
                match box_val.downcast_ref::<T>() {
                    None => panic!("Something went terribly wrong! An Event is in the wrong SimState TypeId bin!"),
                    Some(val) => Some(val)
                }
            }
        }
    }

    /// Sets an `Event` object on the current state. The previous `Event`
    /// of this type (if any) will be replaced with the new `Event`
    /// 
    /// # Arguments
    /// * `event` - `Event` object to set
    /// 
    /// returns previously stored `Event` or `None`
    pub fn set_state<T: Event>(&mut self, event: T) -> Option<Box<dyn Event>> {
        let type_id = TypeId::of::<T>();
        self.state.insert(type_id, Box::new(event))
    }
    
    /// Checks whether an `Event` exists in this state for a given `Event` type
    /// 
    /// returns `true` if it exists or `false` otherwise
    pub fn has_state<T: Event>(&mut self) -> bool {
        let type_id = TypeId::of::<T>();
        self.state.contains_key(&type_id)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};
    use std::cell::{Cell, RefCell};
    use uom::si::f64::Length;
    use uom::si::f64::AmountOfSubstance;
    use uom::si::length::meter;
    use uom::si::amount_of_substance::mole;
    use simple_logger::SimpleLogger;
    use super::SimState;
    use crate::event::Event;
    use crate::event::test::TestEventA;
    use crate::event::test::TestEventB;

    #[test]
    fn test_sim_state() {
        crate::init_test();

        let mut state = SimState::new();

        state.set_state(TestEventA::new(Length::new::<meter>(0.0)));
        assert_eq!(true, state.has_state::<TestEventA>());
        assert_eq!(false, state.has_state::<TestEventB>());
        
        state.set_state(TestEventB::new(AmountOfSubstance::new::<mole>(0.0)));
        assert_eq!(true, state.has_state::<TestEventB>());

        let evt_a = state.get_state::<TestEventA>().take().unwrap();
        assert_eq!(Length::new::<meter>(0.0), evt_a.len)
    }
}