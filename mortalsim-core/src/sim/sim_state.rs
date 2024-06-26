use crate::event::Event;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SimState {
    /// Internal storage mechanism for `SimState` objects
    state: HashMap<TypeId, Arc<dyn Event>>,
    /// Keep track of any Events which have been tainted
    tainted_states: HashSet<TypeId>,
}

impl SimState {
    /// Creates a new `SimState` object
    pub fn new() -> SimState {
        SimState {
            state: HashMap::new(),
            tainted_states: HashSet::new(),
        }
    }

    /// Retrieves a typed reference to an `Event` in this state
    ///
    /// returns a `&E` or `None` if no `Event` of this type has been set
    pub fn get_state<T: Event>(&self) -> Option<&T> {
        self.state
            .get(&TypeId::of::<T>())?
            .downcast_ref::<T>()
    }
    
    /// Retrieves a typed reference to an `Event` in this state
    ///
    /// returns a `&E` or `None` if no `Event` of this type has been set
    pub fn get_state_arc<T: Event>(&self) -> Option<Arc<T>> {
        self.state
            .get(&TypeId::of::<T>())?
            .clone()
            .downcast_arc::<T>().ok()
    }

    /// Retrieves a dyn `Event` in this state
    ///
    /// returns a cloned `Arc<E>` or `None` if no `Event` of this type has been set
    pub fn get_dyn_state(&self, type_id: &TypeId) -> Option<&Arc<dyn Event>> {
        Some(self.state.get(&type_id)?)
    }

    /// Checks whether an `Event` exists in this state for a given `Event` type
    ///
    /// returns `true` if it exists or `false` otherwise
    pub fn has_state<T: Event>(&self) -> bool {
        self.state.contains_key(&TypeId::of::<T>())
    }

    /// Adds an Event to the state given it's TypeId
    ///
    /// ### Arguments
    /// * `type_key` - type of the `Event` object
    /// * `event`    - owned `Event` object to set
    ///
    /// returns previously stored `Event` or `None`
    pub(super) fn put_state(&mut self, event: Arc<dyn Event>) -> Option<Arc<dyn Event>> {
        self.tainted_states.insert(event.type_id());
        self.state.insert(event.type_id(), event)
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
            self.put_state(other.get_dyn_state(type_key).unwrap().clone());
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
                    if std::ptr::addr_eq(&**local_rc as *const dyn Event, &**evt_rc as *const dyn Event) {
                        return;
                    }
                }
                None => {}
            }
            self.put_state(evt_rc.clone());
        }
    }
}


mod tests {
    use super::SimState;
    use crate::event::test::TestEventA;
    use crate::event::test::TestEventB;
    use crate::units::base::Amount;
    use crate::units::base::Distance;
    use std::any::TypeId;
    use std::collections::hash_set::HashSet;

    #[test]
    fn test_sim_state() {
        let mut state = SimState::new();

        state.set_state(TestEventA::new(Distance::from_m(0.0)));
        assert_eq!(true, state.has_state::<TestEventA>());
        assert_eq!(false, state.has_state::<TestEventB>());

        let mut test_set_1 = HashSet::new();
        test_set_1.insert(TypeId::of::<TestEventA>());
        assert_eq!(true, &test_set_1 == state.get_tainted());

        state.set_state(TestEventB::new(Amount::from_mol(0.0)));
        assert_eq!(true, state.has_state::<TestEventB>());

        test_set_1.insert(TypeId::of::<TestEventB>());
        assert_eq!(true, &test_set_1 == state.get_tainted());

        let evt_a = state.get_state::<TestEventA>().take().unwrap();
        assert_eq!(Distance::from_m(0.0), evt_a.len)
    }
}
