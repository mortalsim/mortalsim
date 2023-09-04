use super::{CoreConnector, SimComponent};
use crate::event::Event;
use crate::hub::event_transformer::{EventTransformer, TransformerItem};
use crate::hub::EventHub;
use crate::sim::{SimState, TimeManager};
use crate::util::id_gen::IdType;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Rc, Weak};
use std::sync::Arc;

pub struct CoreComponentInitializer {
    /// Input events for the associated component
    input_events: HashSet<TypeId>,
    /// Output events for the associated component
    output_events: HashSet<TypeId>,
    /// Notifications pending from the last run of the component
    pub(crate) pending_notifies: Vec<(i32, Box<dyn Event>)>,
    /// Transforms pending from the last run of the component
    pub(crate) pending_transforms: Vec<Box<dyn EventTransformer>>,
    /// Default event state from the component
    pub(crate) initial_outputs: Vec<Box<dyn Event>>,
}

impl CoreComponentInitializer {
    pub fn new() -> CoreComponentInitializer {
        CoreComponentInitializer {
            input_events: HashSet::new(),
            output_events: HashSet::new(),
            pending_notifies: Vec::new(),
            pending_transforms: Vec::new(),
            initial_outputs: Vec::new(),
        }
    }

    /// Registers the associated `CoreComponent` to `run` whenever the
    /// provided `Event` is modified on the `Sim`.
    ///
    /// ### Arguments
    /// * `default` - Default `Event` value when one isn't provided by another module
    pub fn notify<E: Event>(&mut self, default: E) {
        self.notify_prioritized::<E>(0, default);
    }

    /// Registers the associated `CoreComponent` to `run` whenever the
    /// provided `Event` is modified on the `Sim` with a given priority value.
    ///
    /// ### Arguments
    /// * `priority` - Notify order priority for this registration
    /// * `default` - Default `Event` value when one isn't provided by another module
    pub fn notify_prioritized<E: Event>(&mut self, priority: i32, default: E) {
        let type_key = TypeId::of::<E>();

        // If this event type has already been registered as an output, panic
        if self.output_events.contains(&type_key) {
            panic!("Modules cannot register notifications for Events they are producing! This could cause an infinite loop.")
        }

        self.input_events.insert(type_key);
        self.pending_notifies.push((priority, Box::new(default)))
    }

    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim`.
    ///
    /// ### Arguments
    /// * `handler` - Function to modify the `Event`
    pub fn transform<E: Event>(&mut self, handler: impl FnMut(&mut E) + 'static) {
        self.pending_transforms
            .push(Box::new(TransformerItem::new(handler)))
    }

    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim` with a given priority value.
    ///
    /// ### Arguments
    /// * `priority` - Transformation order priority for this registration
    /// * `handler` - Function to modify the `Event`
    pub fn transform_prioritized<E: Event>(
        &mut self,
        priority: i32,
        handler: impl FnMut(&mut E) + 'static,
    ) {
        self.pending_transforms
            .push(Box::new(TransformerItem::new_prioritized(
                handler, priority,
            )))
    }

    /// Sets an `Event` as the initial state on the `Sim`
    ///
    /// ### Arguments
    /// * `event` - `Event` instance to set on initial state
    pub fn set_output<E: Event>(&mut self, initial_value: E) {
        let type_key = TypeId::of::<E>();

        // If this event type has already been registered as an output, panic
        if self.input_events.contains(&type_key) {
            panic!("Modules cannot produce Events they are notifying on! This could cause an infinite loop.")
        }

        self.output_events.insert(type_key);
        self.initial_outputs.push(Box::new(initial_value))
    }
}

#[cfg(test)]
pub mod test {
    use crate::event::test::TestEventA;
    use crate::units::base::Distance;

    use super::CoreComponentInitializer;

    fn basic_event() -> TestEventA {
        TestEventA::new(Distance::from_m(1.0))
    }

    fn test_transformer(evt: &mut TestEventA) {
        evt.len += Distance::from_m(1.0);
    }

    #[test]
    fn test_init() {
        CoreComponentInitializer::new();
    }
    
    #[test]
    fn test_notify() {
        let mut initializer = CoreComponentInitializer::new();
        initializer.notify(basic_event());
    }
    
    #[test]
    fn test_notify_with_priority() {
        let mut initializer = CoreComponentInitializer::new();
        initializer.notify(basic_event());
        initializer.notify_prioritized(1, basic_event());
    }
    
    #[test]
    fn test_transform() {
        let mut initializer = CoreComponentInitializer::new();

        // Should accept both static functions and closures
        initializer.transform(test_transformer);
        initializer.transform(|evt: &mut TestEventA| {
            evt.len += Distance::from_m(2.0)
        });
    }
    
    #[test]
    fn test_transform_with_priority() {
        let mut initializer = CoreComponentInitializer::new();

        // Should accept both static functions and closures
        initializer.transform_prioritized(1, |evt: &mut TestEventA| {
            evt.len += Distance::from_m(2.0)
        });
    }
    
    #[test]
    fn test_output() {
        let mut initializer = CoreComponentInitializer::new();
        initializer.set_output(basic_event())
    }
    
    #[test]
    #[should_panic]
    fn test_notify_err() {
        let mut initializer = CoreComponentInitializer::new();
        initializer.notify(basic_event());
        initializer.set_output(basic_event())
    }
    
    #[test]
    #[should_panic]
    fn test_output_err() {
        let mut initializer = CoreComponentInitializer::new();
        initializer.set_output(basic_event());
        initializer.notify(basic_event())
    }
}
