pub(crate) mod connector;
pub(crate) mod initializer;
use crate::sim::component::SimComponent;
use crate::sim::organism::Organism;
pub use connector::CoreConnector;
pub use initializer::CoreInitializer;

/// Trait to implement for `Core` simulation components.
/// 
/// Example:
/// ```
/// use mortalsim_core::event::Event;
/// use mortalsim_core::units::base::Distance;
/// use mortalsim_core::sim::component::{ComponentRegistry, SimComponent};
/// use mortalsim_core::sim::layer::core::{CoreInitializer, CoreConnector, CoreComponent};
/// use mortalsim_core::sim::Organism;
/// 
/// #[derive(Debug)]
/// struct ExampleEventA {
///     len: Distance<f64>
/// };
/// impl Event for ExampleEventA {}
/// 
/// #[derive(Debug)]
/// struct ExampleEventB;
/// impl Event for ExampleEventB {}
/// 
/// struct ExampleComponentA<O: Organism> {
///     connector: CoreConnector<O>,
/// }
/// impl<O: Organism> ExampleComponentA<O> {
///     pub fn new() -> Self {
///         Self { 
///             connector: CoreConnector::new()
///         }
///     }
/// }
/// impl<O: Organism> CoreComponent<O> for ExampleComponentA<O> {
///     fn core_connector(&mut self) -> &mut CoreConnector<O> {
///         &mut self.connector
///     }
///     fn core_init(&mut self, initializer: &mut CoreInitializer<O>) {
///         initializer.notify::<ExampleEventB>();
///         initializer.transform(|evt: &mut ExampleEventA| {
///             evt.len += Distance::from_m(3.0);
///         });
///     }
/// }
/// 
/// impl<O: Organism> SimComponent<O> for ExampleComponentA<O> {
///     fn id(&self) -> &'static str {
///         "ExampleComponentA"
///     }
///     fn attach(self, registry: &mut ComponentRegistry<O>) {
///         registry.add_core_component(self);
///     }
///     fn run(&mut self) {
///         if let Some(evt_b) = self.connector.get::<ExampleEventB>() {
///             println!("{:?} emitted", evt_b)
///         }
///         else {
///             println!("ExampleEventB not emitted")
///         }
///     }
/// }
/// ```
pub trait CoreComponent<O: Organism>: SimComponent<O> {
    /// Initializes the module. Should register any `Event` objects to listen for
    /// and set initial state.
    ///
    /// ### Arguments
    /// * `initializer` - Helper object for initializing the module
    fn core_init(&mut self, initializer: &mut CoreInitializer<O>);

    /// Used by the Sim to retrieve a mutable reference to this module's
    /// CoreConnector, which tracks module interactions
    ///
    /// ### returns
    /// CoreConnector to interact with the rest of the simulation
    fn core_connector(&mut self) -> &mut CoreConnector<O>;
}


pub mod test {
    use super::CoreComponent;
    use super::{CoreConnector, CoreInitializer};
    use crate::event::test::{TestEventA, TestEventB};
    use crate::sim::component::registry::ComponentRegistry;
    use crate::sim::component::SimComponent;
    use crate::sim::organism::test::TestSim;
    use crate::sim::organism::Organism;
    use crate::sim::organism::test::TestOrganism;
    use crate::units::base::Amount;
    use crate::units::base::Distance;
    use std::any::TypeId;

    pub struct TestComponentA<O: Organism> {
        connector: CoreConnector<O>,
    }
    impl<O: Organism> TestComponentA<O> {
        pub fn new() -> Self {
            Self {
                connector: CoreConnector::new(),
            }
        }
    }
    impl<O: Organism> CoreComponent<O> for TestComponentA<O> {
        fn core_connector(&mut self) -> &mut CoreConnector<O> {
            &mut self.connector
        }
        fn core_init(&mut self, initializer: &mut CoreInitializer<O>) {
            initializer.notify::<TestEventA>();
            initializer.notify::<TestEventB>();
            initializer.transform(|evt: &mut TestEventA| {
                evt.len = Distance::from_m(3.0);
            });
        }
    }

    impl<O: Organism> SimComponent<O> for TestComponentA<O> {
        fn id(&self) -> &'static str {
            "TestComponentA"
        }
        fn attach(self, registry: &mut ComponentRegistry<O>) {
            registry.add_core_component(self);
        }
        fn run(&mut self) {
            if let Some(evt_a) = self.connector.get::<TestEventA>() {
                assert_eq!(evt_a.len, Distance::from_m(3.0));

                log::debug!(
                    "Trigger Events: {:?}",
                    self.connector.trigger_events().collect::<Vec<&TypeId>>()
                );
            }
        }
    }

    pub struct TestComponentB<O: Organism> {
        connector: CoreConnector<O>,
    }
    impl<O: Organism> TestComponentB<O> {
        pub fn new() -> Self {
            Self {
                connector: CoreConnector::new(),
            }
        }

        pub fn transform_b(evt: &mut TestEventB) {
            evt.amt = evt.amt + Amount::from_mol(0.0);
        }
    }
    impl<O: Organism> CoreComponent<O> for TestComponentB<O> {
        fn core_init(&mut self, initializer: &mut CoreInitializer<O>) {
            initializer.notify::<TestEventA>();
            initializer.transform(Self::transform_b);
        }
        fn core_connector(&mut self) -> &mut CoreConnector<O> {
            &mut self.connector
        }
    }

    impl<O: Organism> SimComponent<O> for TestComponentB<O> {
        fn id(&self) -> &'static str {
            "TestComponentB"
        }
        fn attach(self, registry: &mut ComponentRegistry<O>) {
            registry.add_core_component(self);
        }
        fn run(&mut self) {
            if let Some(evt_a) = self.connector.get::<TestEventA>() {
                assert_eq!(evt_a.len, Distance::from_m(3.0));

                log::debug!(
                    "Trigger Events: {:?}",
                    self.connector.trigger_events().collect::<Vec<&TypeId>>()
                );
            }
        }
    }

    #[test]
    fn test_component() {
        let mut component = TestComponentA::new();
        let mut initializer = CoreInitializer::new();
        CoreComponent::<TestOrganism>::core_init(&mut component, &mut initializer);

        assert!(initializer.pending_notifies.len() == 2);
        assert!(initializer.pending_transforms.len() == 1);
    }
}
