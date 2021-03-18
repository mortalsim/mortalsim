use std::sync::Arc;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::any::TypeId;
use uuid::Uuid;
use crate::event::Event;
use anyhow::{Result, Error};

#[derive(Debug)]
pub struct SimState {
    state_id: Uuid,
    /// Internal storage mechanism for `SimState` objects
    state: HashMap<TypeId, Arc<dyn Event>>,
    /// Keep track of any Events which have been tainted
    tainted_states: HashSet<TypeId>,
}

impl SimState {
    /// Creates a new `SimState` object
    pub fn new() -> SimState {
        SimState {
            state_id: Uuid::new_v4(),
            state: HashMap::new(),
            tainted_states: HashSet::new(),
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
    
    /// Checks whether an `Event` exists in this state for a given `Event` type
    /// 
    /// returns `true` if it exists or `false` otherwise
    pub fn has_state<T: Event>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.state.contains_key(&type_id)
    }
    
    /// Retrieves an Rc to the current `Event` of a given type in this state
    /// 
    /// returns an `Arc<Event>` or `None` if no `Event` of this type has been set
    pub(super) fn get_state_ref(&self, type_id: &TypeId) -> Option<Arc<dyn Event>> {
        match self.state.get(&type_id) {
            None => None,
            Some(rc_val) => Some(rc_val.clone())
        }
    }

    /// Adds an Event to the state given it's TypeId
    /// 
    /// ### Arguments
    /// * `type_key` - type of the `Event` object
    /// * `event`    - owned `Event` object to set
    /// 
    /// returns previously stored `Event` or `None`
    pub(super) fn put_state(&mut self, type_key: TypeId, event: Arc<dyn Event>) -> Option<Arc<dyn Event>> {
        self.tainted_states.insert(type_key);
        self.state.insert(type_key, event)
    }
    
    /// Sets an `Event` object on the current state. The previous `Event`
    /// of this type (if any) will be replaced with the new `Event`
    /// 
    /// ### Arguments
    /// * `event` - `Event` object to set
    /// 
    /// returns previously stored `Event` or `None`
    pub(super) fn set_state<T: Event>(&mut self, event: T) -> Option<Arc<dyn Event>> {
        let type_id = TypeId::of::<T>();
        self.tainted_states.insert(type_id);
        self.state.insert(type_id, Arc::new(event))
    }
    
    /// Sets an `Event` object on the current state, without tainting.
    /// The previous `Event` of this type (if any) will be replaced with
    /// the new `Event`
    /// 
    /// ### Arguments
    /// * `event` - `Event` object to set
    /// 
    /// returns previously stored `Event` or `None`
    pub(super) fn set_state_quiet<T: Event>(&mut self, event: T) -> Option<Arc<dyn Event>> {
        let type_id = TypeId::of::<T>();
        self.state.insert(type_id, Arc::new(event))
    }

    /// Retrieves the `Set` of tainted `Event` `TypeId`s on this `State`
    pub(super) fn get_tainted(&self) -> &HashSet<TypeId> {
        &self.tainted_states
    }

    /// Resets the tainted status on all Event types
    pub(super) fn clear_taint(&mut self) {
        self.tainted_states.clear();
    }

    /// Merges tainted Events from the target `SimState` to this one, overwriting
    /// any matching `Events` which exist in this `SimState`.
    /// 
    /// ### Arguments
    /// * `other` - Other `SimState` to merge into this one
    pub fn merge_tainted(&mut self, other: &Self) {
        for type_key in other.tainted_states.iter() {
            self.put_state(type_key.clone(), other.get_state_ref(type_key).unwrap());
        }
    }
    
    /// Merges all Events from the target `SimState` to this one, overwriting
    /// any matching `Events` which exist in this `SimState`.
    /// 
    /// ### Arguments
    /// * `other` - Other `SimState` to merge into this one
    pub fn merge_all(&mut self, other: &Self) {
        for (type_key, evt_rc) in other.state.iter() {
            match self.state.get(type_key) {
                Some(local_rc) => {
                    // If both refs point to the same object, ignore it
                    if &**local_rc as *const dyn Event == &**evt_rc as *const dyn Event {
                        return
                    }
                }
                None => {}
            }
            self.put_state(type_key.clone(), evt_rc.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};
    use std::cell::{Cell, RefCell};
    use std::collections::hash_set::HashSet;
    use std::any::TypeId;
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
        crate::test::init_test();

        let mut state = SimState::new();

        state.set_state(TestEventA::new(Length::new::<meter>(0.0)));
        assert_eq!(true, state.has_state::<TestEventA>());
        assert_eq!(false, state.has_state::<TestEventB>());

        let mut test_set_1 = HashSet::new();
        test_set_1.insert(TypeId::of::<TestEventA>());
        assert_eq!(true, &test_set_1 == state.get_tainted());
        
        state.set_state(TestEventB::new(AmountOfSubstance::new::<mole>(0.0)));
        assert_eq!(true, state.has_state::<TestEventB>());
        
        test_set_1.insert(TypeId::of::<TestEventB>());
        assert_eq!(true, &test_set_1 == state.get_tainted());

        let evt_a = state.get_state::<TestEventA>().take().unwrap();
        assert_eq!(Length::new::<meter>(0.0), evt_a.len)
    }
}