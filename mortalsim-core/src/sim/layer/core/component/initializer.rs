use crate::event::Event;
use crate::hub::event_transformer::{EventTransformer, TransformerItem};
use crate::sim::Organism;
use crate::IdGenerator;
use crate::IdType;
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

/// Provides `Core` component initialization methods
pub struct CoreInitializer<O: Organism> {
    pd: PhantomData<O>,
    /// Local id generator for transformation registration
    pub(crate) id_gen: IdGenerator,
    /// Notifications pending from the last run of the component
    pub(crate) pending_notifies: Vec<TypeId>,
    /// Transforms pending initial addition
    pub(crate) pending_transforms: Vec<(IdType, Box<dyn EventTransformer>)>,
    /// Default event state from the component
    pub(crate) initial_outputs: Vec<Box<dyn Event>>,
}

impl<O: Organism> CoreInitializer<O> {
    pub fn new() -> Self {
        Self {
            pd: PhantomData,
            id_gen: IdGenerator::new(),
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
    pub fn notify<E: Event>(&mut self) {
        self.pending_notifies.push(TypeId::of::<E>())
    }

    fn register_transform<E: Event>(&mut self, transformer: TransformerItem<'static, E>) -> IdType {
        let local_id = self.id_gen.get_id();

        self.pending_transforms
            .push((local_id, Box::new(transformer)));

        local_id
    }

    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim`.
    ///
    /// ### Arguments
    /// * `handler` - Function to modify the `Event`
    /// 
    /// Returns a registration id for this transformer
    pub fn transform<E: Event>(&mut self, handler: impl FnMut(&mut E) + Send + 'static) -> IdType {
        self.register_transform(TransformerItem::new(handler))
    }

    /// Registers a transformation function whenever the indicated `Event` is
    /// emitted for the correspoinding `Sim` with a given priority value.
    ///
    /// ### Arguments
    /// * `priority` - Transformation order priority for this registration
    /// * `handler` - Function to modify the `Event`
    /// 
    /// Returns a registration id for this transformer
    pub fn transform_prioritized<E: Event>(
        &mut self,
        priority: i32,
        handler: impl FnMut(&mut E) + Send + 'static,
    ) -> IdType {
        self.register_transform(TransformerItem::new_prioritized(
            handler, priority,
        ))
    }

    /// Sets an `Event` as the initial state on the `Sim`
    ///
    /// ### Arguments
    /// * `event` - `Event` instance to set on initial state
    pub fn set_output<E: Event>(&mut self, initial_value: E) {
        self.initial_outputs.push(Box::new(initial_value))
    }
}

#[cfg(test)]
pub mod test {
    use crate::event::test::{TestEventA, TestEventB};
    use crate::sim::organism::test::{TestOrganism, TestSim};
    use crate::units::base::Distance;

    use super::CoreInitializer;

    fn basic_event() -> TestEventA {
        TestEventA::new(Distance::from_m(1.0))
    }

    fn test_transformer(evt: &mut TestEventA) {
        evt.len += Distance::from_m(1.0);
    }

    #[test]
    fn test_init() {
        CoreInitializer::<TestOrganism>::new();
    }

    #[test]
    fn test_notify() {
        let mut initializer = CoreInitializer::<TestOrganism>::new();
        initializer.notify::<TestEventA>();
    }

    #[test]
    fn test_transform() {
        let mut initializer = CoreInitializer::<TestOrganism>::new();

        // Should accept both static functions and closures
        initializer.transform(test_transformer);
        initializer.transform(|evt: &mut TestEventA| evt.len += Distance::from_m(2.0));
    }

    #[test]
    fn test_transform_with_priority() {
        let mut initializer = CoreInitializer::<TestOrganism>::new();

        // Should accept both static functions and closures
        initializer
            .transform_prioritized(1, |evt: &mut TestEventA| evt.len += Distance::from_m(2.0));
    }

    #[test]
    fn test_output() {
        let mut initializer = CoreInitializer::<TestOrganism>::new();
        initializer.set_output(basic_event())
    }

}
